use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper, r2d2::ConnectionManager,
};
use r2d2::Pool;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;

use crate::schema::{channel, channel_participant, device, message, message_payload, user};
use crate::{
    AppError, Channel, ChannelInfo, ChannelParticipant, ChatMessage, Device, InboundChatMessage,
    MessagePayload, NewChatMessage, NewMessagePayload, OutboundChatMessage, User, WsEvent,
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
        channel_id: Uuid,
    ) -> Result<ChannelInfo, AppError>;
    async fn get_user_channels(&self, user: &User) -> Result<Vec<Channel>, AppError>;
    async fn get_channel_history(
        &self,
        user: &User,
        channel_id: Uuid,
        device_id: Uuid,
        after: Option<Uuid>,
    ) -> Result<Vec<OutboundChatMessage>, AppError>;

    // Message operations
    async fn save_message(
        &self,
        user: &User,
        message: InboundChatMessage,
    ) -> Result<(ChatMessage, Vec<MessagePayload>), AppError>;

    // Real-time delivery
    async fn register_device(&self, device_id: Uuid, tx: mpsc::Sender<WsEvent>);
    async fn unregister_device(&self, device_id: Uuid);
    async fn get_broadcaster(&self, user: &User) -> broadcast::Sender<WsEvent>;
    async fn get_broadcaster_for_device(&self, device_id: Uuid) -> Option<mpsc::Sender<WsEvent>>;
    async fn notify_user(&self, user: &User, event: WsEvent);
}

pub struct DbMessageRelayService {
    pool: Pool<ConnectionManager<PgConnection>>,
    user_websockets: Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsEvent>>>>,
    device_websockets: Arc<RwLock<HashMap<Uuid, mpsc::Sender<WsEvent>>>>,
}

impl DbMessageRelayService {
    #[must_use]
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self {
            pool,
            user_websockets: Arc::default(),
            device_websockets: Arc::default(),
        }
    }

    fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, AppError> {
        self.pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))
    }

    async fn get_channel_participants(&self, channel_id: Uuid) -> Result<Vec<User>, AppError> {
        let mut conn = self.get_conn()?;

        let users = tokio::task::spawn_blocking(move || {
            channel_participant::table
                .filter(channel_participant::channel_id.eq(channel_id))
                .inner_join(user::table.on(user::id.eq(channel_participant::user_id)))
                .select(User::as_select())
                .load(&mut conn)
        })
        .await??;

        Ok(users)
    }
}

#[async_trait]
impl MessageRelayService for DbMessageRelayService {
    #[tracing::instrument(skip(self))]
    async fn get_channel_info(
        &self,
        user: &User,
        channel_id: Uuid,
    ) -> Result<ChannelInfo, AppError> {
        let mut conn = self.get_conn()?;

        let participants = self.get_channel_participants(channel_id).await?;

        if !participants.contains(user) {
            return Err(AppError::Unauthorized);
        }

        let devices = device::table
            .inner_join(user::table.on(device::user_id.eq(user::id)))
            .filter(user::id.eq_any(participants.iter().map(|u| u.id)))
            .distinct()
            .select(Device::as_select())
            .load(&mut conn)?;

        Ok(ChannelInfo {
            channel_id,
            participants,
            devices,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_user_channels(&self, user: &User) -> Result<Vec<Channel>, AppError> {
        let mut conn = self.get_conn()?;

        let user_id = user.id;
        let channels = tokio::task::spawn_blocking(move || {
            channel_participant::table
                .filter(channel_participant::user_id.eq(user_id))
                .inner_join(channel::table.on(channel::id.eq(channel_participant::channel_id)))
                .select(Channel::as_select())
                .load(&mut conn)
        })
        .await??;

        Ok(channels)
    }

    async fn get_channel_history(
        &self,
        user: &User,
        channel_id: Uuid,
        device_id: Uuid,
        after: Option<Uuid>,
    ) -> Result<Vec<OutboundChatMessage>, AppError> {
        let participants = self.get_channel_participants(channel_id).await?;
        if !participants.contains(user) {
            return Err(AppError::Unauthorized);
        }

        let mut conn = self.get_conn()?;

        let history = tokio::task::spawn_blocking(move || {
            let mut query = message::table
                .inner_join(
                    message_payload::table.on(message::id
                        .eq(message_payload::message_id)
                        .and(message_payload::recipient_device_id.eq(device_id))),
                )
                .filter(message::channel_id.eq(channel_id))
                .select((
                    message::id,
                    message::sender_device_id,
                    message::channel_id,
                    message::sender_id,
                    message_payload::ciphertext,
                    message::created,
                    message_payload::is_pre_key,
                ))
                .order(message::id.asc())
                .into_boxed();

            if let Some(after) = after {
                query = query.filter(message::id.gt(after));
            }

            query.load::<OutboundChatMessage>(&mut conn)
        })
        .await??;

        Ok(history)
    }

    async fn create_channel_between(
        &self,
        sender: &User,
        recipient: &User,
    ) -> Result<ChannelInfo, AppError> {
        let mut conn = self.get_conn()?;

        if sender == recipient {
            return Err(AppError::UserError(
                "can't make chat with yourself".to_string(),
            ));
        }

        let channel = diesel::insert_into(channel::table)
            .default_values()
            .returning(Channel::as_returning())
            .get_result(&mut conn)?;

        let participant1 = ChannelParticipant {
            channel_id: channel.id,
            user_id: sender.id,
        };

        let participant2 = ChannelParticipant {
            channel_id: channel.id,
            user_id: recipient.id,
        };

        diesel::insert_into(channel_participant::table)
            .values(&[participant1, participant2])
            .execute(&mut conn)?;

        let channel_info = self.get_channel_info(sender, channel.id).await?;
        Ok(channel_info)
    }

    async fn save_message(
        &self,
        user: &User,
        message: InboundChatMessage,
    ) -> Result<(ChatMessage, Vec<MessagePayload>), AppError> {
        let mut conn = self.get_conn()?;

        let users = self.get_channel_participants(message.channel_id).await?;

        if !users.contains(user) {
            return Err(AppError::Unauthorized);
        }

        let new_message = NewChatMessage::from_inbound(user, &message);
        let payloads = message
            .payloads
            .into_iter()
            .map(|m| m.into_new_message(message.message_id))
            .collect::<Result<Vec<NewMessagePayload>, _>>()?;

        let message = diesel::insert_into(message::table)
            .values(&new_message)
            .returning(ChatMessage::as_returning())
            .get_result(&mut conn)?;

        let payloads = diesel::insert_into(message_payload::table)
            .values(&payloads)
            .returning(MessagePayload::as_returning())
            .load(&mut conn)?;

        Ok((message, payloads))
    }

    async fn register_device(&self, device_id: Uuid, device_tx: mpsc::Sender<WsEvent>) {
        let mut device_websockets = self.device_websockets.write().await;
        device_websockets.insert(device_id, device_tx);
    }

    async fn unregister_device(&self, device_id: Uuid) {
        let mut device_websockets = self.device_websockets.write().await;
        device_websockets.remove(&device_id);
    }

    async fn get_broadcaster(&self, user: &User) -> broadcast::Sender<WsEvent> {
        self.user_websockets
            .write()
            .await
            .entry(user.id)
            .or_insert_with(|| broadcast::Sender::new(128))
            .clone()
    }

    async fn get_broadcaster_for_device(&self, device_id: Uuid) -> Option<mpsc::Sender<WsEvent>> {
        let device_websockets = self.device_websockets.read().await;
        device_websockets.get(&device_id).cloned()
    }

    #[tracing::instrument(skip(self, event))]
    async fn notify_user(&self, user: &User, event: WsEvent) {
        let broadcaster = self.get_broadcaster(user).await;
        match broadcaster.send(event) {
            Ok(n) => tracing::debug!("notified user {} ({n} receivers)", user.id),
            Err(e) => tracing::warn!("failed to notify user {}: {e}", user.id),
        }
    }
}
