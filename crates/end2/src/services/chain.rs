use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use miner::{Chain, DeviceRecord};

use crate::{AppError, Device, InboundDevice, User};

use super::DeviceKeyService;

pub struct ChainDeviceKeyService {
    chain: Arc<RwLock<Chain>>,
}

impl ChainDeviceKeyService {
    #[must_use]
    pub const fn new(chain: Arc<RwLock<Chain>>) -> Self {
        Self { chain }
    }
}

fn device_from_record(r: &DeviceRecord) -> Device {
    Device {
        id: r.device_id,
        user_id: r.user_id,
        ed25519: Some(r.ed25519.to_vec()),
        x25519: Some(r.x25519.to_vec()),
    }
}

#[async_trait]
#[allow(clippy::significant_drop_tightening)]
impl DeviceKeyService for ChainDeviceKeyService {
    async fn new_device_for(&self, _user: &User) -> Result<Device, AppError> {
        Err(AppError::UserError(
            "device registration is handled on-chain by clients".into(),
        ))
    }

    async fn get_device(&self, user: &User, device_id: Uuid) -> Result<Device, AppError> {
        let (record_user_id, device) = {
            let chain = self.chain.read().await;
            let record = chain
                .state()
                .get_device(device_id)
                .ok_or_else(|| AppError::QueryFailed("device not found on chain".into()))?;
            (record.user_id, device_from_record(record))
        };

        if record_user_id != user.id {
            return Err(AppError::QueryFailed("device not found".into()));
        }

        Ok(device)
    }

    async fn get_all_devices(&self, user: &User) -> Result<Vec<Device>, AppError> {
        let devices = {
            let chain = self.chain.read().await;
            chain
                .state()
                .get_user_devices(user.id)
                .into_iter()
                .map(device_from_record)
                .collect()
        };
        Ok(devices)
    }

    async fn set_device_keys(
        &self,
        _user: &User,
        _device_id: Uuid,
        _keys: InboundDevice,
    ) -> Result<Device, AppError> {
        Err(AppError::UserError(
            "key updates are handled on-chain by clients".into(),
        ))
    }
}
