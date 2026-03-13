use async_trait::async_trait;

use crate::{AppError, Device, DeviceId, InboundDevice, User};

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
}