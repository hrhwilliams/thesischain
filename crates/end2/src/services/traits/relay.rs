use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc};

use crate::{
    AppError, Channel, ChannelId, ChannelInfo, ChatMessage, DeviceId, InboundChatMessage,
    MessageId, MessagePayload, OutboundChatMessage, User, WsEvent,
};

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
