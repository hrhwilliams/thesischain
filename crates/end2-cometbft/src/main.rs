mod store;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jmt::{JellyfishMerkleTree, KeyHash, storage::TreeUpdateBatch};
use rocksdb::WriteBatch;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tendermint_abci::{Application, ServerBuilder};
use tendermint_proto::abci::{
    Event, EventAttribute, ExecTxResult, RequestCheckTx, RequestFinalizeBlock, RequestInfo,
    RequestQuery, ResponseCheckTx, ResponseCommit, ResponseFinalizeBlock, ResponseInfo,
    ResponseQuery,
};

use crate::store::{META_APP_HASH, META_HEIGHT, Store};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InboundAuthorization {
    pub authorizing_device_id: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize)]
struct KeyPayload {
    user_hash: String,
    device_id: String,
    x25519: String,
    ed25519: String,
    signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    authorization: Option<InboundAuthorization>,
}

fn decode_key32(s: &str) -> Result<[u8; 32], &'static str> {
    BASE64_STANDARD_NO_PAD
        .decode(s)
        .map_err(|_| "key is not valid base64")?
        .try_into()
        .map_err(|_| "key must be 32 bytes")
}

#[derive(Deserialize)]
struct KeyUploadTx {
    payload: KeyPayload,
    signature: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct DeviceKeys {
    x25519: [u8; 32],
    ed25519: [u8; 32],
}

// Pending state produced by finalize_block, written atomically in commit.
struct Pending {
    height: u64,
    app_hash: [u8; 32],
    // (rocksdb key, JSON(DeviceKeys)) — written into rocksdb on commit.
    writes: Vec<(Vec<u8>, Vec<u8>)>,
    tree_update: TreeUpdateBatch,
}

#[derive(Clone)]
struct KeyDirectoryApp {
    store: Arc<Store>,
    verifying_key: Arc<VerifyingKey>,
    pending: Arc<Mutex<Option<Pending>>>,
}

impl KeyDirectoryApp {
    fn new(verifying_key: VerifyingKey, store: Store) -> Self {
        Self {
            store: Arc::new(store),
            verifying_key: Arc::new(verifying_key),
            pending: Arc::new(Mutex::new(None)),
        }
    }
}

fn verify_tx(bytes: &[u8], key: &VerifyingKey) -> Result<KeyPayload, &'static str> {
    let tx: KeyUploadTx = serde_json::from_slice(bytes).map_err(|_| "invalid JSON")?;

    let sig_bytes = BASE64_STANDARD_NO_PAD
        .decode(&tx.signature)
        .map_err(|_| "signature is not valid base64")?;

    let sig_bytes: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| "signature must be 64 bytes")?;

    let signature = Signature::from_bytes(&sig_bytes);
    let msg = serde_json::to_vec(&tx.payload).map_err(|_| "failed to serialize payload")?;
    key.verify(&msg, &signature)
        .map_err(|_| "signature verification failed")?;

    Ok(tx.payload)
}

fn check_tx_err(log: &str) -> ResponseCheckTx {
    ResponseCheckTx {
        code: 1,
        log: log.to_owned(),
        ..Default::default()
    }
}

fn err_query(msg: &str) -> ResponseQuery {
    ResponseQuery {
        code: 1,
        log: msg.to_owned(),
        ..Default::default()
    }
}

impl Application for KeyDirectoryApp {
    fn info(&self, _req: RequestInfo) -> ResponseInfo {
        let height = self.store.last_height();
        let app_hash = self.store.last_app_hash();
        ResponseInfo {
            data: "end2-cometbft".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            app_version: 1,
            last_block_height: height as i64,
            last_block_app_hash: app_hash.into(),
        }
    }

    fn check_tx(&self, req: RequestCheckTx) -> ResponseCheckTx {
        match verify_tx(&req.tx, &self.verifying_key) {
            Ok(_) => ResponseCheckTx::default(),
            Err(msg) => check_tx_err(msg),
        }
    }

    fn finalize_block(&self, req: RequestFinalizeBlock) -> ResponseFinalizeBlock {
        let mut tx_results: Vec<ExecTxResult> = Vec::with_capacity(req.txs.len());

        // Overlay of this block's writes keyed by rocksdb key, for hash computation
        // and to detect add-vs-update within the same block.
        let mut overlay: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        let mut tree_updates: Vec<(KeyHash, Option<Vec<u8>>)> = Vec::with_capacity(req.txs.len());

        for raw in &req.txs {
            let payload = match verify_tx(raw, &self.verifying_key) {
                Ok(p) => p,
                Err(msg) => {
                    tx_results.push(ExecTxResult {
                        code: 1,
                        log: msg.to_owned(),
                        ..Default::default()
                    });
                    continue;
                }
            };

            let (x25519, ed25519) = match (
                decode_key32(&payload.x25519),
                decode_key32(&payload.ed25519),
            ) {
                (Ok(x), Ok(e)) => (x, e),
                _ => {
                    tx_results.push(ExecTxResult {
                        code: 1,
                        log: "invalid key encoding".to_owned(),
                        ..Default::default()
                    });
                    continue;
                }
            };

            let new_keys = DeviceKeys { x25519, ed25519 };
            let user_hash_hex = payload.user_hash.clone();
            let device_id = payload.device_id.clone();

            let rk = Store::device_key(&user_hash_hex, &device_id);
            let existed = overlay.contains_key(&rk)
                || self.store.get_device(&user_hash_hex, &device_id).is_some();
            let event_type = if existed { "key_update" } else { "key_add" };

            let value_json = serde_json::to_vec(&new_keys).expect("DeviceKeys json");
            overlay.insert(rk.clone(), value_json.clone());

            let key_hash = KeyHash::with::<Sha256>(&rk);
            let val_hash = Sha256::digest(&value_json).to_vec();
            tree_updates.push((key_hash, Some(val_hash)));

            tx_results.push(ExecTxResult {
                events: vec![Event {
                    r#type: event_type.to_owned(),
                    attributes: vec![
                        EventAttribute {
                            key: "user_hash".to_owned(),
                            value: user_hash_hex,
                            index: true,
                        },
                        EventAttribute {
                            key: "device_id".to_owned(),
                            value: device_id,
                            index: true,
                        },
                    ],
                }],
                ..Default::default()
            });
        }

        let version = req.height as u64;

        let tree = JellyfishMerkleTree::<_, Sha256>::new(self.store.as_ref());
        let (root_hash, tree_update) = tree
            .put_value_set(tree_updates, version)
            .expect("JMT put_value_set failed");

        let writes: Vec<(Vec<u8>, Vec<u8>)> = overlay.into_iter().collect();
        *self.pending.lock().expect("lock pending") = Some(Pending {
            height: version,
            app_hash: root_hash.0,
            writes,
            tree_update,
        });

        ResponseFinalizeBlock {
            tx_results,
            app_hash: root_hash.0.to_vec().into(),
            ..Default::default()
        }
    }

    fn commit(&self) -> ResponseCommit {
        let pending = self.pending.lock().unwrap().take();
        if let Some(p) = pending {
            let mut batch = WriteBatch::default();

            for (k, v) in p.writes {
                batch.put_cf(self.store.cf_device(), k, v);
            }

            self.store.write_tree_update(&mut batch, p.tree_update);

            batch.put(META_HEIGHT, p.height.to_le_bytes());
            batch.put(META_APP_HASH, p.app_hash);

            let mut wo = rocksdb::WriteOptions::default();
            wo.set_sync(true);
            self.store
                .db
                .write_opt(batch, &wo)
                .expect("rocksdb commit write failed");
        }
        ResponseCommit::default()
    }

    /// Supported query paths. Store key is `hex(user_hash)`, a 64-char hex string.
    ///
    /// `"device"` - fetch one device's keys.
    /// `data` = `"<hex_user_hash>:<device_id>"` as UTF-8, then hex-encoded for the RPC.
    /// Returns `DeviceKeys` as JSON in `response.value` (RPC base64-encodes it).
    ///
    /// `"devices"` - fetch all devices for a user.
    /// `data` = `"<hex_user_hash>"` as UTF-8, then hex-encoded for the RPC.
    /// Returns `HashMap<device_id, DeviceKeys>` as JSON.
    fn query(&self, req: RequestQuery) -> ResponseQuery {
        match req.path.as_str() {
            "device" => {
                let raw = String::from_utf8_lossy(&req.data);
                let Some((user_hash_hex, device_id)) = raw.split_once(':') else {
                    return err_query("data must be '<hex_user_hash>:<device_id>'");
                };
                match self.store.get_device(user_hash_hex, device_id) {
                    Some(bytes) => ResponseQuery {
                        value: bytes.into(),
                        ..Default::default()
                    },
                    None => err_query(&format!(
                        "no device '{device_id}' for hash '{user_hash_hex}'"
                    )),
                }
            }

            "devices" => {
                let user_hash_hex = String::from_utf8_lossy(&req.data);
                let v = self.store.iter_user_devices(user_hash_hex.as_ref());
                if v.is_empty() {
                    return err_query(&format!("no devices for hash '{user_hash_hex}'"));
                }
                let mut map = serde_json::Map::with_capacity(v.len());
                for (id, bytes) in v {
                    let val: serde_json::Value =
                        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
                    map.insert(id, val);
                }
                ResponseQuery {
                    value: serde_json::to_vec(&serde_json::Value::Object(map))
                        .unwrap_or_default()
                        .into(),
                    ..Default::default()
                }
            }

            other => err_query(&format!(
                "unknown path '{other}': use 'device' or 'devices'"
            )),
        }
    }
}

fn main() -> Result<(), tendermint_abci::Error> {
    dotenvy::dotenv().ok();

    let pubkey_b64 = std::env::var("ABCI_SERVER_PUBKEY").expect("ABCI_SERVER_PUBKEY must be set");
    let pubkey_bytes = BASE64_STANDARD_NO_PAD
        .decode(pubkey_b64)
        .expect("ABCI_SERVER_PUBKEY is not valid base64");
    let pubkey_bytes: [u8; 32] = pubkey_bytes
        .try_into()
        .expect("ABCI_SERVER_PUBKEY must be 32 bytes");
    let verifying_key =
        VerifyingKey::from_bytes(&pubkey_bytes).expect("invalid ed25519 public key");

    let port = std::env::var("ABCI_PORT").unwrap_or_else(|_| "26658".into());
    let db_path = std::env::var("ABCI_DB_PATH").unwrap_or_else(|_| "./abci-data".into());

    let store = Store::open(&db_path).expect("failed to open rocksdb");
    let app = KeyDirectoryApp::new(verifying_key, store);

    ServerBuilder::default()
        .bind(format!("0.0.0.0:{port}"), app)
        .expect("failed to bind ABCI server")
        .listen()
}
