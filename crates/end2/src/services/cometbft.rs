use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_STANDARD, prelude::BASE64_STANDARD_NO_PAD};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper, r2d2::ConnectionManager,
};
use ed25519_dalek::{Signer, SigningKey};
use r2d2::Pool;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vodozemac::{Curve25519PublicKey, Ed25519PublicKey};

use crate::{
    AppError, Device, DeviceId, DeviceKeyService, InboundDevice, NewDevice, User,
    schema::{device, user as user_table},
};

// These structs mirror the ones in end2-cometbft/src/main.rs and must stay in sync.
// Binary fields are unpadded base64 strings.
#[derive(Serialize, Deserialize)]
struct KeyPayload {
    user_hash: String,
    device_id: String,
    x25519: String,
    ed25519: String,
}

#[derive(Serialize)]
struct KeyUploadTx {
    payload: KeyPayload,
    signature: String,
}

#[derive(Deserialize)]
struct AbciDeviceKeys {
    x25519: [u8; 32],
    ed25519: [u8; 32],
}

#[derive(Serialize)]
struct JsonRpcRequest<P: Serialize> {
    jsonrpc: &'static str,
    id: u32,
    method: &'static str,
    params: P,
}

#[derive(Deserialize)]
struct JsonRpcResponse<R> {
    result: R,
}

#[derive(Deserialize)]
struct BroadcastResult {
    check_tx: TxResult,
    tx_result: TxResult,
}

#[derive(Deserialize)]
struct TxResult {
    code: u32,
    log: String,
}

#[derive(Deserialize)]
struct AbciQueryResult {
    response: AbciQueryResponse,
}

#[derive(Deserialize)]
struct AbciQueryResponse {
    code: u32,
    log: String,
    value: Option<String>,
}

#[derive(Serialize)]
struct BroadcastParams {
    tx: String,
}

#[derive(Serialize)]
struct AbciQueryParams {
    path: String,
    data: String,
    prove: bool,
}

#[derive(Clone)]
pub struct CometBftDeviceKeyService {
    http: Client,
    rpc_url: String,
    signing_key: Arc<SigningKey>,
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl CometBftDeviceKeyService {
    #[must_use]
    pub fn new(
        rpc_url: String,
        signing_key: Arc<SigningKey>,
        pool: Pool<ConnectionManager<PgConnection>>,
    ) -> Self {
        Self {
            http: Client::new(),
            rpc_url,
            signing_key,
            pool,
        }
    }

    fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, AppError> {
        self.pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))
    }

    fn user_hash(user: &User) -> [u8; 32] {
        Sha256::digest(format!("{}", user.id)).into()
    }

    async fn broadcast_tx(&self, tx: &KeyUploadTx) -> Result<(), AppError> {
        let tx_bytes = serde_json::to_vec(tx).map_err(|e| AppError::ValueError(e.to_string()))?;

        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "broadcast_tx_commit",
            params: BroadcastParams {
                tx: BASE64_STANDARD.encode(&tx_bytes),
            },
        };

        let res: JsonRpcResponse<BroadcastResult> = self
            .http
            .post(&self.rpc_url)
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::ValueError(e.to_string()))?
            .json()
            .await
            .map_err(|e| AppError::ValueError(e.to_string()))?;

        if res.result.check_tx.code != 0 {
            return Err(AppError::ValueError(format!(
                "check_tx rejected: {}",
                res.result.check_tx.log
            )));
        }

        if res.result.tx_result.code != 0 {
            return Err(AppError::ValueError(format!(
                "finalize_block rejected: {}",
                res.result.tx_result.log
            )));
        }

        Ok(())
    }

    async fn abci_query(&self, path: &str, data: &[u8]) -> Result<Vec<u8>, AppError> {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "abci_query",
            params: AbciQueryParams {
                path: path.to_owned(),
                data: hex::encode(data),
                prove: false,
            },
        };

        let res: JsonRpcResponse<AbciQueryResult> = self
            .http
            .post(&self.rpc_url)
            .json(&req)
            .send()
            .await
            .map_err(|e| AppError::ValueError(e.to_string()))?
            .json()
            .await
            .map_err(|e| AppError::ValueError(e.to_string()))?;

        if res.result.response.code != 0 {
            return Err(AppError::UserError(res.result.response.log));
        }

        let value = res
            .result
            .response
            .value
            .ok_or_else(|| AppError::ValueError("empty response value".into()))?;

        BASE64_STANDARD
            .decode(&value)
            .map_err(|e| AppError::InvalidB64(e.to_string()))
    }
}

#[async_trait]
impl DeviceKeyService for CometBftDeviceKeyService {
    #[tracing::instrument(skip(self))]
    async fn new_device_for(&self, user: &User) -> Result<Device, AppError> {
        let mut conn = self.get_conn()?;
        let new_device = NewDevice {
            user_id: user.id,
            x25519: None,
            ed25519: None,
        };

        let device = diesel::insert_into(device::table)
            .values(&new_device)
            .returning(Device::as_returning())
            .get_result(&mut conn)?;

        Ok(device)
    }

    #[tracing::instrument(skip(self))]
    async fn get_device(&self, user: &User, device_id: DeviceId) -> Result<Device, AppError> {
        let user_hash = Self::user_hash(user);
        let user_hash_hex = hex::encode(user_hash);
        let query = format!("{user_hash_hex}:{device_id}");

        let value = self.abci_query("device", query.as_bytes()).await?;

        let keys: AbciDeviceKeys =
            serde_json::from_slice(&value).map_err(|e| AppError::ValueError(e.to_string()))?;

        Ok(Device {
            id: device_id,
            user_id: user.id,
            x25519: Some(keys.x25519.to_vec()),
            ed25519: Some(keys.ed25519.to_vec()),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_devices(&self, user: &User) -> Result<Vec<Device>, AppError> {
        let user_hash = Self::user_hash(user);
        let user_hash_hex = hex::encode(user_hash);

        let value = match self.abci_query("devices", user_hash_hex.as_bytes()).await {
            Ok(v) => v,
            Err(AppError::UserError(_)) => return Ok(vec![]),
            Err(e) => return Err(e),
        };

        let map: HashMap<String, AbciDeviceKeys> =
            serde_json::from_slice(&value).map_err(|e| AppError::ValueError(e.to_string()))?;

        let devices = map
            .into_iter()
            .filter_map(|(id_str, keys)| {
                DeviceId::try_from(id_str.as_str()).ok().map(|id| Device {
                    id,
                    user_id: user.id,
                    x25519: Some(keys.x25519.to_vec()),
                    ed25519: Some(keys.ed25519.to_vec()),
                })
            })
            .collect();

        Ok(devices)
    }

    #[tracing::instrument(skip(self))]
    async fn set_device_keys(
        &self,
        user: &User,
        device_id: DeviceId,
        keys: InboundDevice,
    ) -> Result<Device, AppError> {
        let x25519 = Curve25519PublicKey::from_base64(&keys.x25519)
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;
        let ed25519 = Ed25519PublicKey::from_base64(&keys.ed25519)
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        let x25519_bytes: &[u8; 32] = x25519
            .as_bytes()
            .try_into()
            .map_err(|_| AppError::InvalidKeySize)?;
        let ed25519_bytes: &[u8; 32] = ed25519
            .as_bytes()
            .try_into()
            .map_err(|_| AppError::InvalidKeySize)?;

        let payload = KeyPayload {
            user_hash: hex::encode(Self::user_hash(user)),
            device_id: device_id.to_string(),
            x25519: BASE64_STANDARD_NO_PAD.encode(x25519_bytes),
            ed25519: BASE64_STANDARD_NO_PAD.encode(ed25519_bytes),
        };

        let msg = serde_json::to_vec(&payload).map_err(|e| AppError::ValueError(e.to_string()))?;
        let sig = self.signing_key.sign(&msg);

        // Send keys to CometBFT
        self.broadcast_tx(&KeyUploadTx {
            payload,
            signature: BASE64_STANDARD_NO_PAD.encode(sig.to_bytes()),
        })
        .await?;

        // Store keys in DB as well
        let x25519_db = x25519.as_bytes().to_vec();
        let ed25519_db = ed25519.as_bytes().to_vec();
        let user = user.clone();
        let mut conn = self.get_conn()?;

        let device = tokio::task::spawn_blocking(move || {
            diesel::update(device::table)
                .filter(device::id.eq(device_id).and(device::user_id.eq(user.id)))
                .set((device::x25519.eq(x25519_db), device::ed25519.eq(ed25519_db)))
                .returning(Device::as_returning())
                .get_result(&mut conn)
        })
        .await??;

        Ok(device)
    }

    #[tracing::instrument(skip(self))]
    async fn get_valid_users(&self) -> Result<usize, AppError> {
        let mut conn = self.get_conn()?;
        let users = tokio::task::spawn_blocking(move || {
            user_table::table.select(User::as_select()).load(&mut conn)
        })
        .await??;

        let mut have_devices = 0_usize;
        for user in &users {
            if self
                .get_all_devices(user)
                .await
                .is_ok_and(|d| !d.is_empty())
            {
                have_devices += 1;
            }
        }

        Ok(have_devices)
    }
}
