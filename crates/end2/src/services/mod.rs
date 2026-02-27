mod auth;
mod chain;
mod device;
mod otk;
mod relay;

use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc};

pub use auth::*;
pub use chain::*;
pub use device::*;
pub use otk::*;
pub use relay::*;

use crate::{
    AppError, Channel, ChannelId, ChannelInfo, ChatMessage, Device, DeviceId, InboundChatMessage,
    InboundDevice, InboundDiscordInfo, InboundOtks, InboundUser, LoginError, MessageId,
    MessagePayload, Otk, OutboundChatMessage, RegistrationError, User, UserId, WsEvent,
};

/// How the backend authenticates users and stores/distributes user info
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn register_user(&self, inbound: InboundUser) -> Result<User, RegistrationError>;
    async fn login(&self, username: &str, password: &str) -> Result<User, LoginError>;
    async fn login_with_discord(&self, info: &InboundDiscordInfo) -> Result<User, LoginError>;
    async fn register_with_discord(
        &self,
        info: InboundDiscordInfo,
    ) -> Result<User, RegistrationError>;
    async fn link_account(
        &self,
        user: &User,
        info: InboundDiscordInfo,
    ) -> Result<(), RegistrationError>;
    async fn get_user_info(&self, user_id: UserId) -> Result<Option<User>, AppError>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn get_user_by_discord_id(&self, discord_id: i64) -> Result<Option<User>, AppError>;
    async fn change_nickname(&self, user: &User, nickname: &str) -> Result<(), AppError>;
    async fn get_known_users(&self, user: &User) -> Result<Vec<User>, AppError>;
}

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

#[async_trait]
pub trait MessageRelayService: Send + Sync {
    // Channel operations
    async fn create_channel_between(
        &self,
        sender: &User,
        recipient: &User,
    ) -> Result<ChannelInfo, AppError>;
    async fn get_channel_info(
        &self,
        user: &User,
        channel_id: ChannelId,
    ) -> Result<ChannelInfo, AppError>;
    async fn get_user_channels(&self, user: &User) -> Result<Vec<Channel>, AppError>;
    async fn get_channel_history(
        &self,
        user: &User,
        channel_id: ChannelId,
        device_id: DeviceId,
        after: Option<MessageId>,
    ) -> Result<Vec<OutboundChatMessage>, AppError>;

    // Message operations
    async fn save_message(
        &self,
        user: &User,
        message: InboundChatMessage,
    ) -> Result<(ChatMessage, Vec<MessagePayload>), AppError>;

    // Real-time delivery
    async fn register_device(&self, device_id: DeviceId, tx: mpsc::Sender<WsEvent>);
    async fn unregister_device(&self, device_id: DeviceId);
    async fn get_broadcaster(&self, user: &User) -> broadcast::Sender<WsEvent>;
    async fn get_broadcaster_for_device(
        &self,
        device_id: DeviceId,
    ) -> Option<mpsc::Sender<WsEvent>>;
    async fn notify_user(&self, user: &User, event: WsEvent);
}
