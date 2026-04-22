mod store;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rocksdb::WriteBatch;
use sha2::{Digest, Sha256};
use tendermint_abci::{Application, ServerBuilder};
use tendermint_proto::abci as proto;

use crate::store::{META_APP_HASH, META_HEIGHT, Store};

#[derive(serde::Serialize, serde::Deserialize)]
struct KeyPayload {
    user_hash: String,
    device_id: String,
    x25519: String,
    ed25519: String,
    signature: String,
}

fn decode_key32(s: &str) -> Result<[u8; 32], &'static str> {
    BASE64_STANDARD_NO_PAD
        .decode(s)
        .map_err(|_| "key is not valid base64")?
        .try_into()
        .map_err(|_| "key must be 32 bytes")
}

#[derive(serde::Deserialize)]
struct KeyUploadTx {
    payload: KeyPayload,
    signature: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
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

fn check_tx_err(log: &str) -> proto::ResponseCheckTx {
    proto::ResponseCheckTx {
        code: 1,
        log: log.to_owned(),
        ..Default::default()
    }
}

fn err_query(msg: &str) -> proto::ResponseQuery {
    proto::ResponseQuery {
        code: 1,
        log: msg.to_owned(),
        ..Default::default()
    }
}

impl Application for KeyDirectoryApp {
    fn info(&self, _req: proto::RequestInfo) -> proto::ResponseInfo {
        let height = self.store.last_height();
        let app_hash = self.store.last_app_hash();
        proto::ResponseInfo {
            data: "end2-cometbft".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            app_version: 1,
            last_block_height: height as i64,
            last_block_app_hash: app_hash.into(),
        }
    }

    fn check_tx(&self, req: proto::RequestCheckTx) -> proto::ResponseCheckTx {
        match verify_tx(&req.tx, &self.verifying_key) {
            Ok(_) => proto::ResponseCheckTx::default(),
            Err(msg) => check_tx_err(msg),
        }
    }

    fn finalize_block(&self, req: proto::RequestFinalizeBlock) -> proto::ResponseFinalizeBlock {
        let mut tx_results: Vec<proto::ExecTxResult> = Vec::with_capacity(req.txs.len());

        // Overlay of this block's writes keyed by rocksdb key, for hash computation
        // and to detect add-vs-update within the same block.
        let mut overlay: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();

        for raw in &req.txs {
            let payload = match verify_tx(raw, &self.verifying_key) {
                Ok(p) => p,
                Err(msg) => {
                    tx_results.push(proto::ExecTxResult {
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
                    tx_results.push(proto::ExecTxResult {
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
            overlay.insert(rk, value_json);

            tx_results.push(proto::ExecTxResult {
                events: vec![proto::Event {
                    r#type: event_type.to_owned(),
                    attributes: vec![
                        proto::EventAttribute {
                            key: "user_hash".to_owned(),
                            value: user_hash_hex,
                            index: true,
                        },
                        proto::EventAttribute {
                            key: "device_id".to_owned(),
                            value: device_id,
                            index: true,
                        },
                    ],
                }],
                ..Default::default()
            });
        }

        // Compute app_hash over (current rocksdb state ∪ overlay), sorted by key.
        // Merge: rocksdb iterator is already in sorted order; BTreeMap too.
        let app_hash = compute_app_hash(&self.store, &overlay);

        let writes: Vec<(Vec<u8>, Vec<u8>)> = overlay.into_iter().collect();
        *self.pending.lock().unwrap() = Some(Pending {
            height: req.height as u64,
            app_hash,
            writes,
        });

        proto::ResponseFinalizeBlock {
            tx_results,
            app_hash: app_hash.to_vec().into(),
            ..Default::default()
        }
    }

    fn commit(&self) -> proto::ResponseCommit {
        let pending = self.pending.lock().unwrap().take();
        if let Some(p) = pending {
            let mut batch = WriteBatch::default();
            for (k, v) in p.writes {
                batch.put(k, v);
            }
            batch.put(META_HEIGHT, p.height.to_le_bytes());
            batch.put(META_APP_HASH, p.app_hash);

            let mut wo = rocksdb::WriteOptions::default();
            wo.set_sync(true);
            self.store
                .db
                .write_opt(batch, &wo)
                .expect("rocksdb commit write failed");
        }
        proto::ResponseCommit::default()
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
    fn query(&self, req: proto::RequestQuery) -> proto::ResponseQuery {
        match req.path.as_str() {
            "device" => {
                let raw = String::from_utf8_lossy(&req.data);
                let Some((user_hash_hex, device_id)) = raw.split_once(':') else {
                    return err_query("data must be '<hex_user_hash>:<device_id>'");
                };
                match self.store.get_device(user_hash_hex, device_id) {
                    Some(bytes) => proto::ResponseQuery {
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
                proto::ResponseQuery {
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

// Deterministic sha256 over all (key, value) pairs in sorted rocksdb-key order,
// with the current block's overlay applied on top.
fn compute_app_hash(store: &Store, overlay: &BTreeMap<Vec<u8>, Vec<u8>>) -> [u8; 32] {
    let mut hasher = Sha256::new();

    // Merge rocksdb entries with this block's overlay into one sorted stream,
    // overlay wins on key collision.
    let db_entries = store.all_devices();
    let mut db_iter = db_entries.into_iter().peekable();
    let mut ov_iter = overlay.iter().peekable();

    loop {
        let next = match (db_iter.peek(), ov_iter.peek()) {
            (None, None) => break,
            (Some(_), None) => {
                let (k, v) = db_iter.next().unwrap();
                (k, v)
            }
            (None, Some(_)) => {
                let (k, v) = ov_iter.next().unwrap();
                (k.clone(), v.clone())
            }
            (Some((dk, _)), Some((ok, _))) => {
                use std::cmp::Ordering::*;
                match dk.as_slice().cmp(ok.as_slice()) {
                    Less => {
                        let (k, v) = db_iter.next().unwrap();
                        (k, v)
                    }
                    Greater => {
                        let (k, v) = ov_iter.next().unwrap();
                        (k.clone(), v.clone())
                    }
                    Equal => {
                        let _ = db_iter.next();
                        let (k, v) = ov_iter.next().unwrap();
                        (k.clone(), v.clone())
                    }
                }
            }
        };
        let (k, v) = next;
        hasher.update(&(k.len() as u32).to_le_bytes());
        hasher.update(&k);
        hasher.update(&(v.len() as u32).to_le_bytes());
        hasher.update(&v);
    }

    hasher.finalize().into()
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
