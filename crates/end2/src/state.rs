use std::sync::Arc;

use diesel::{PgConnection, r2d2::ConnectionManager};
use ed25519_dalek::SigningKey;
use r2d2::Pool;
use secrecy::SecretString;
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::{RwLock, broadcast, mpsc};

use crate::{
    AppError, Channel, ChannelId, ChannelInfo, ChatMessage, Device, DeviceId, InboundChatMessage,
    InboundDevice, InboundDiscordInfo, InboundOtks, InboundUser, LoginError, MessageId,
    MessagePayload, OAuthHandler, Otk, OutboundChatMessage, RegistrationError, SessionId, User,
    UserId, WebSession, WebSessionService, WsEvent,
    services::{AuthService, DeviceKeyService, MessageRelayService, OtkService},
};

#[derive(Clone)]
pub enum AppEvent {}

#[derive(Clone)]
pub struct AppState<
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService,
> {
    pub auth: A,
    pub device_keys: D,
    pub otks: O,
    pub relay: R,
    pub web_sessions: W,
    pub signing_key: Arc<SigningKey>,
    pub miners: Arc<RwLock<Vec<miner::MinerInfo>>>,
    pool: Pool<ConnectionManager<PgConnection>>,
    /// Sends AppEvents to subscribers
    broadcaster: broadcast::Sender<AppEvent>,
}

impl<
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService,
> AppState<A, D, O, R, W>
{
    #[must_use]
    pub fn new(
        auth: A,
        device_keys: D,
        otks: O,
        relay: R,
        web_sessions: W,
        pool: Pool<ConnectionManager<PgConnection>>,
        signing_key: SigningKey,
    ) -> Self {
        let (broadcaster, _) = broadcast::channel(256);

        Self {
            auth,
            device_keys,
            otks,
            relay,
            web_sessions,
            signing_key: Arc::new(signing_key),
            miners: Arc::new(RwLock::new(Vec::new())),
            pool,
            broadcaster,
        }
    }

    pub(crate) fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, AppError> {
        self.pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.broadcaster.subscribe()
    }

    // --- Auth ---

    pub async fn register_user(&self, inbound: InboundUser) -> Result<User, RegistrationError> {
        self.auth.register_user(inbound).await
    }

    pub async fn login(&self, username: &str, password: SecretString) -> Result<User, LoginError> {
        self.auth.login(username, password).await
    }

    pub async fn login_with_discord(&self, info: &InboundDiscordInfo) -> Result<User, LoginError> {
        self.auth.login_with_discord(info).await
    }

    pub async fn register_with_discord(
        &self,
        info: InboundDiscordInfo,
    ) -> Result<User, RegistrationError> {
        self.auth.register_with_discord(info).await
    }

    pub async fn link_account(
        &self,
        user: &User,
        info: InboundDiscordInfo,
    ) -> Result<(), RegistrationError> {
        self.auth.link_account(user, info).await
    }

    pub async fn get_user_info(&self, user_id: UserId) -> Result<Option<User>, AppError> {
        self.auth.get_user_info(user_id).await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        self.auth.get_user_by_username(username).await
    }

    pub async fn get_user_by_discord_id(&self, discord_id: i64) -> Result<Option<User>, AppError> {
        self.auth.get_user_by_discord_id(discord_id).await
    }

    pub async fn change_nickname(&self, user: &User, nickname: &str) -> Result<(), AppError> {
        self.auth.change_nickname(user, nickname).await
    }

    pub async fn get_known_users(&self, user: &User) -> Result<Vec<User>, AppError> {
        self.auth.get_known_users(user).await
    }

    pub fn get_oauth_handler(&self, service: &str) -> Option<&OAuthHandler> {
        self.auth.get_oauth_handler(service)
    }

    // --- Device keys/Identity keys ---

    pub async fn new_device_for(&self, user: &User) -> Result<Device, AppError> {
        self.device_keys.new_device_for(user).await
    }

    pub async fn get_device(&self, user: &User, device_id: DeviceId) -> Result<Device, AppError> {
        self.device_keys.get_device(user, device_id).await
    }

    pub async fn get_all_devices(&self, user: &User) -> Result<Vec<Device>, AppError> {
        self.device_keys.get_all_devices(user).await
    }

    pub async fn set_device_keys(
        &self,
        user: &User,
        device_id: DeviceId,
        keys: InboundDevice,
    ) -> Result<Device, AppError> {
        self.device_keys
            .set_device_keys(user, device_id, keys)
            .await
    }

    // --- One-time keys ---

    pub async fn get_otks(&self, device_id: DeviceId) -> Result<Vec<Otk>, AppError> {
        self.otks.get_otks(device_id).await
    }

    pub async fn upload_otks(
        &self,
        user: &User,
        device_id: DeviceId,
        otks: InboundOtks,
    ) -> Result<(), AppError> {
        self.otks.upload_otks(user, device_id, otks).await
    }

    pub async fn get_user_otk(&self, user: &User, device_id: DeviceId) -> Result<Otk, AppError> {
        self.otks.get_user_otk(user, device_id).await
    }

    // --- Message relay ---

    pub async fn create_channel_between(
        &self,
        sender: &User,
        recipient: &User,
    ) -> Result<ChannelInfo, AppError> {
        self.relay.create_channel_between(sender, recipient).await
    }

    pub async fn get_channel_info(
        &self,
        user: &User,
        channel_id: ChannelId,
    ) -> Result<ChannelInfo, AppError> {
        self.relay.get_channel_info(user, channel_id).await
    }

    pub async fn get_user_channels(&self, user: &User) -> Result<Vec<Channel>, AppError> {
        self.relay.get_user_channels(user).await
    }

    pub async fn get_channel_history(
        &self,
        user: &User,
        channel_id: ChannelId,
        device_id: DeviceId,
        after: Option<MessageId>,
    ) -> Result<Vec<OutboundChatMessage>, AppError> {
        self.relay
            .get_channel_history(user, channel_id, device_id, after)
            .await
    }

    pub async fn save_message(
        &self,
        user: &User,
        message: InboundChatMessage,
    ) -> Result<(ChatMessage, Vec<MessagePayload>), AppError> {
        self.relay.save_message(user, message).await
    }

    pub async fn register_device(&self, device_id: DeviceId, tx: mpsc::Sender<WsEvent>) {
        self.relay.register_device(device_id, tx).await;
    }

    pub async fn unregister_device(&self, device_id: DeviceId) {
        self.relay.unregister_device(device_id).await;
    }

    pub async fn get_broadcaster(&self, user: &User) -> broadcast::Sender<WsEvent> {
        self.relay.get_broadcaster(user).await
    }

    pub async fn get_broadcaster_for_device(
        &self,
        device_id: DeviceId,
    ) -> Option<mpsc::Sender<WsEvent>> {
        self.relay.get_broadcaster_for_device(device_id).await
    }

    pub async fn notify_user(&self, user: &User, event: WsEvent) {
        self.relay.notify_user(user, event).await;
    }

    // --- Web sessions ---

    pub async fn new_session(&self) -> Result<WebSession, AppError> {
        self.web_sessions.new_session().await
    }

    pub async fn get_session(&self, session_id: SessionId) -> Result<Option<WebSession>, AppError> {
        self.web_sessions.get_session(session_id).await
    }

    pub async fn insert_into_session<T: Serialize + Send>(
        &self,
        web_session: WebSession,
        key: String,
        value: T,
    ) -> Result<WebSession, AppError> {
        self.web_sessions
            .insert_into_session(web_session, key, value)
            .await
    }

    pub async fn get_from_session<T: DeserializeOwned + Send>(
        &self,
        web_session: &WebSession,
        key: &str,
    ) -> Result<Option<T>, AppError> {
        self.web_sessions.get_from_session(web_session, key).await
    }

    pub async fn remove_from_session<T: DeserializeOwned + Send>(
        &self,
        web_session: WebSession,
        key: &str,
    ) -> Result<Option<(T, WebSession)>, AppError> {
        self.web_sessions
            .remove_from_session(web_session, key)
            .await
    }
}
