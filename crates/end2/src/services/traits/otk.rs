use async_trait::async_trait;

use crate::{AppError, DeviceId, InboundOtks, Otk, User};

/// How the backend distributes per-device one-time keys
#[async_trait]
pub trait OtkService: Send + Sync {
    async fn get_otks(&self, device_id: DeviceId) -> Result<Vec<Otk>, AppError>;
    async fn upload_otks(
        &self,
        user: &User,
        device_id: DeviceId,
        otks: InboundOtks,
    ) -> Result<(), AppError>;
    async fn get_user_otk(&self, user: &User, device_id: DeviceId) -> Result<Otk, AppError>;
}