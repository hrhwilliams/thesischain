use async_trait::async_trait;

use crate::{AppError, Device, DeviceId, HistoricalKey, InboundDevice, User};

/// How the backend stores and distributes long-term device keys
#[async_trait]
pub trait DeviceKeyService: Send + Sync {
    async fn new_device_for(&self, user: &User) -> Result<Device, AppError>;
    async fn get_device(&self, user: &User, device_id: DeviceId) -> Result<Device, AppError>;
    async fn get_all_devices(&self, user: &User) -> Result<Vec<Device>, AppError>;
    async fn set_device_keys(
        &self,
        user: &User,
        device_id: DeviceId,
        keys: InboundDevice,
    ) -> Result<Device, AppError>;

    async fn get_valid_users(&self) -> Result<usize, AppError> {
        Err(AppError::UserError("not supported".into()))
    }

    async fn get_device_key_history(
        &self,
        _user: &User,
        _device_id: DeviceId,
    ) -> Result<Vec<HistoricalKey>, AppError> {
        Err(AppError::UserError("key history not supported by this backend".into()))
    }
}
