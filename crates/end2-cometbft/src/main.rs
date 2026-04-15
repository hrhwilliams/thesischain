use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use tendermint_abci::{Application, ServerBuilder};
use tendermint_proto::abci as proto;

// The signable payload. Serialized to canonical JSON and signed by the server.
// user_hash is hex; x25519, ed25519, and signature are unpadded base64.
// signature is the device self-signature (ed25519 over x25519||ed25519 bytes).
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

// Full tx: payload + base64 ed25519 signature over the JSON of `payload`.
#[derive(serde::Deserialize)]
struct KeyUploadTx {
    payload: KeyPayload,
    signature: String,
}

// What we store per device and return from queries.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct DeviceKeys {
    x25519: [u8; 32],
    ed25519: [u8; 32],
}

#[derive(Clone)]
struct KeyDirectoryApp {
    store: Arc<RwLock<HashMap<String, HashMap<String, DeviceKeys>>>>,
    verifying_key: Arc<VerifyingKey>,
}

impl KeyDirectoryApp {
    fn new(verifying_key: VerifyingKey) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            verifying_key: Arc::new(verifying_key),
        }
    }
}

// Parse and verify a tx. Returns the payload on success.
fn verify_tx(bytes: &[u8], key: &VerifyingKey) -> Result<KeyPayload, &'static str> {
    let tx: KeyUploadTx = serde_json::from_slice(bytes).map_err(|_| "invalid JSON")?;

    let sig_bytes = BASE64_STANDARD_NO_PAD
        .decode(&tx.signature)
        .map_err(|_| "signature is not valid base64")?;

    let sig_bytes: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| "signature must be 64 bytes")?;

    let signature = Signature::from_bytes(&sig_bytes);

    // Signed message = canonical JSON bytes of the payload struct
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
    fn check_tx(&self, req: proto::RequestCheckTx) -> proto::ResponseCheckTx {
        match verify_tx(&req.tx, &self.verifying_key) {
            Ok(_) => proto::ResponseCheckTx::default(),
            Err(msg) => check_tx_err(msg),
        }
    }

    fn finalize_block(&self, req: proto::RequestFinalizeBlock) -> proto::ResponseFinalizeBlock {
        let tx_results = req
            .txs
            .iter()
            .map(|raw| match verify_tx(raw, &self.verifying_key) {
                Ok(payload) => {
                    let (x25519, ed25519) = match (
                        decode_key32(&payload.x25519),
                        decode_key32(&payload.ed25519),
                    ) {
                        (Ok(x), Ok(e)) => (x, e),
                        _ => {
                            return proto::ExecTxResult {
                                code: 1,
                                log: "invalid key encoding".to_owned(),
                                ..Default::default()
                            }
                        }
                    };
                    let new_keys = DeviceKeys { x25519, ed25519 };
                    let user_hash_hex = payload.user_hash.clone();

                    let mut store = self.store.write().unwrap();
                    let devices = store.entry(user_hash_hex.clone()).or_default();
                    let event_type = if devices.contains_key(&payload.device_id) {
                        "key_update"
                    } else {
                        "key_add"
                    };
                    devices.insert(payload.device_id.clone(), new_keys);

                    proto::ExecTxResult {
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
                                    value: payload.device_id,
                                    index: true,
                                },
                            ],
                        }],
                        ..Default::default()
                    }
                }
                Err(msg) => proto::ExecTxResult {
                    code: 1,
                    log: msg.to_owned(),
                    ..Default::default()
                },
            })
            .collect();

        // Compute a deterministic state commitment over all stored keys.
        // Sort entries so the hash is independent of HashMap iteration order.
        let store = self.store.read().unwrap();
        let mut entries: Vec<(&str, &str, &[u8], &[u8])> = store
            .iter()
            .flat_map(|(user_hash, devices)| {
                devices.iter().map(|(device_id, keys)| {
                    (
                        user_hash.as_str(),
                        device_id.as_str(),
                        keys.x25519.as_ref(),
                        keys.ed25519.as_ref(),
                    )
                })
            })
            .collect();
        entries.sort_unstable_by_key(|(u, d, _, _)| (*u, *d));

        let mut hasher = Sha256::new();
        for (user_hash, device_id, x25519, ed25519) in entries {
            hasher.update(user_hash.as_bytes());
            hasher.update(device_id.as_bytes());
            hasher.update(x25519);
            hasher.update(ed25519);
        }

        proto::ResponseFinalizeBlock {
            tx_results,
            app_hash: hasher.finalize().to_vec().into(),
            ..Default::default()
        }
    }

    /// Supported query paths. Store key is `hex(user_hash)`, a 64-char hex string.
    ///
    /// `"device"` - fetch one device's keys.
    /// `data` = `"<hex_user_hash>:<device_id>"` as UTF-8, then hex-encoded for the RPC.
    /// Returns `DeviceKeys` as JSON in `response.value` (RPC base64-encodes it).
    ///
    /// ```sh
    /// HASH=$(echo -n 'deadbeef...64hexchars:dev_01' | xxd -p)
    /// curl "http://comet:26657/abci_query?path=\"device\"&data=0x$HASH"
    /// # response.value (base64-decoded): {"x25519":[1,2,...],"ed25519":[1,2,...]}
    /// ```
    ///
    /// `"devices"` - fetch all devices for a user.
    /// `data` = `"<hex_user_hash>"` as UTF-8, then hex-encoded for the RPC.
    /// Returns `HashMap<device_id, DeviceKeys>` as JSON.
    ///
    /// ```sh
    /// HASH=$(echo -n 'deadbeef...64hexchars' | xxd -p)
    /// curl "http://comet:26657/abci_query?path=\"devices\"&data=0x$HASH"
    /// # response.value (base64-decoded): {"dev_01":{"x25519":[...],"ed25519":[...]},...}
    /// ```
    fn query(&self, req: proto::RequestQuery) -> proto::ResponseQuery {
        let store = self.store.read().unwrap();

        match req.path.as_str() {
            "device" => {
                let raw = String::from_utf8_lossy(&req.data);
                let Some((user_hash_hex, device_id)) = raw.split_once(':') else {
                    return err_query("data must be '<hex_user_hash>:<device_id>'");
                };

                match store.get(user_hash_hex).and_then(|d| d.get(device_id)) {
                    Some(keys) => proto::ResponseQuery {
                        value: serde_json::to_vec(keys).unwrap_or_default().into(),
                        ..Default::default()
                    },
                    None => err_query(&format!(
                        "no device '{device_id}' for hash '{user_hash_hex}'"
                    )),
                }
            }

            "devices" => {
                let user_hash_hex = String::from_utf8_lossy(&req.data);
                match store.get(user_hash_hex.as_ref()) {
                    Some(devices) => proto::ResponseQuery {
                        value: serde_json::to_vec(devices).unwrap_or_default().into(),
                        ..Default::default()
                    },
                    None => err_query(&format!("no devices for hash '{user_hash_hex}'")),
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

    ServerBuilder::default()
        .bind(
            format!("0.0.0.0:{port}"),
            KeyDirectoryApp::new(verifying_key),
        )
        .expect("failed to bind ABCI server")
        .listen()
}
