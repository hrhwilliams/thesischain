use std::collections::HashMap;
use std::num::ParseIntError;
use std::sync::Arc;

use argon2::password_hash::Encoding;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl, SelectableHelper, r2d2::ConnectionManager,
};
use diesel::{JoinOnDsl, alias};
use ed25519_dalek::Signature;
use r2d2::Pool;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;
use vodozemac::Curve25519PublicKey;

use crate::models::User;
use crate::schema::{channel, device, user, web_session};
use crate::{
    AppError, Channel, ChannelInfo, ChannelResponse, ChatMessage, Device, InboundChatMessage, InboundDevice, InboundDiscordInfo, InboundOtks, InboundUser, LoginError, MessagePayload, NewChannel, NewChatMessage, NewDevice, NewDiscordInfo, NewMessagePayload, NewOtk, NewUser, OAuthHandler, Otk, OutboundChatMessage, OutboundDevice, RegistrationError, WebSession, WsEvent, discord_info, is_valid_nickname, is_valid_username, message, message_payload, one_time_key
};

#[derive(Clone)]
pub struct AppState {
    pub oauth: OAuthHandler,
    pool: Pool<ConnectionManager<PgConnection>>,
    user_websockets: Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsEvent>>>>,
    device_websockets: Arc<RwLock<HashMap<Uuid, mpsc::Sender<WsEvent>>>>,
}

impl AppState {
    #[must_use]
    pub fn new(oauth: OAuthHandler, pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self {
            oauth,
            pool,
            user_websockets: Arc::default(),
            device_websockets: Arc::default(),
        }
    }

    pub async fn register_device(&self, device_id: Uuid, device_tx: mpsc::Sender<WsEvent>) {
        let mut device_websockets = self.device_websockets.write().await;
        device_websockets.insert(device_id, device_tx);
    }

    pub async fn unregister_device(&self, device_id: Uuid) {
        let mut device_websockets = self.device_websockets.write().await;
        device_websockets.remove(&device_id);
    }

    pub async fn get_broadcaster(&self, user: &User) -> broadcast::Sender<WsEvent> {
        let mut user_websockets = self.user_websockets.write().await;
        let sender = user_websockets
            .entry(user.id)
            .or_insert(broadcast::Sender::new(128));
        sender.clone()
    }

    pub async fn get_broadcaster_for_device(
        &self,
        device_id: Uuid,
    ) -> Option<mpsc::Sender<WsEvent>> {
        let device_websockets = self.device_websockets.read().await;
        device_websockets.get(&device_id).cloned()
    }

    #[tracing::instrument(skip(self))]
    pub async fn notify_user(&self, user: &User, event: WsEvent) {
        tracing::info!("sending event");
        let broadcaster = self.get_broadcaster(user).await;
        broadcaster.send(event);
    }

    #[tracing::instrument(skip(self))]
    pub async fn new_session(&self) -> Result<WebSession, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let session = diesel::insert_into(web_session::table)
            .default_values()
            .returning(WebSession::as_returning())
            .get_result(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(session)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_session(&self, session_id: Uuid) -> Result<Option<WebSession>, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let session = web_session::table
            .filter(web_session::id.eq(session_id))
            .select(WebSession::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(session)
    }

    pub async fn insert_into_session<T: Serialize>(
        &self,
        web_session: WebSession,
        key: String,
        value: T,
    ) -> Result<WebSession, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let blob = match web_session.blob {
            Value::Object(mut m) => {
                m.insert(
                    key,
                    serde_json::to_value(value).map_err(|e| AppError::ValueError(e.to_string()))?,
                );
                Value::Object(m)
            }
            _ => unreachable!("only blob should be stored in web_session table"),
        };

        let web_session = diesel::update(web_session::table)
            .filter(web_session::id.eq(web_session.id))
            .set(web_session::blob.eq(blob))
            .get_result(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(web_session)
    }

    pub async fn get_from_session<T: DeserializeOwned>(
        &self,
        web_session: &WebSession,
        key: &str,
    ) -> Result<Option<T>, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let blob = web_session::table
            .filter(web_session::id.eq(web_session.id))
            .select(web_session::blob)
            .get_result(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        let value = match blob {
            Value::Object(m) => m
                .get(key)
                .cloned()
                .and_then(|v| serde_json::from_value(v).ok()),
            _ => unreachable!("only blob should be stored in web_session table"),
        };

        Ok(value)
    }

    pub async fn remove_from_session<T: DeserializeOwned>(
        &self,
        web_session: WebSession,
        key: &str,
    ) -> Result<Option<(T, WebSession)>, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let (value, blob) = match web_session.blob {
            Value::Object(mut m) => {
                let value = m.remove(key).and_then(|v| serde_json::from_value(v).ok());
                (value, Value::Object(m))
            }
            _ => unreachable!("only blob should be stored in web_session table"),
        };

        if let Some(value) = value {
            let web_session = diesel::update(web_session::table)
                .filter(web_session::id.eq(web_session.id))
                .set(web_session::blob.eq(blob))
                .get_result(&mut conn)
                .map_err(|e| AppError::QueryFailed(e.to_string()))?;

            Ok(Some((value, web_session)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user(&self, user_id: Uuid) -> Result<Option<User>, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = user::table
            .filter(user::id.eq(user_id))
            .select(User::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_by_username(&self, username: String) -> Result<Option<User>, AppError> {
        tracing::info!("querying for user");
        let pool = self.pool.clone();
        let mut conn: r2d2::PooledConnection<ConnectionManager<PgConnection>> =
            pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = user::table
            .filter(user::username.eq(username))
            .select(User::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_by_discord_id(&self, discord_id: i64) -> Result<Option<User>, AppError> {
        tracing::info!("querying for user");
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = discord_info::table
            .inner_join(user::table)
            .filter(discord_info::discord_id.eq(discord_id))
            .select(User::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    pub async fn login(&self, username: String, password: String) -> Result<User, LoginError> {
        tracing::info!("logging in user");
        let user = self
            .get_user_by_username(username)
            .await
            .map_err(|e| LoginError::InternalError(e))?
            .ok_or(LoginError::NoSuchUser)?;

        let user_password = user.password.as_ref().ok_or(LoginError::NoPassword)?;

        let password_hash = PasswordHash::parse(user_password, Encoding::B64)
            .map_err(|e| AppError::ArgonError(e.to_string()))?;

        let is_correct = Argon2::default()
            .verify_password(password.as_bytes(), &password_hash)
            .is_ok();

        if is_correct {
            Ok(user)
        } else {
            Err(LoginError::InvalidPassword)
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn login_with_discord(
        &self,
        inbound_discord_info: &InboundDiscordInfo,
    ) -> Result<User, LoginError> {
        tracing::info!("logging in user via discord");
        let discord_id = inbound_discord_info
            .id
            .parse()
            .map_err(|e: ParseIntError| LoginError::InvalidDiscordId(e.to_string()))?;
        self.get_user_by_discord_id(discord_id)
            .await
            .map_err(|e| LoginError::InternalError(e))?
            .ok_or(LoginError::NoSuchUser)
    }

    pub async fn register_with_discord(
        &self,
        inbound: InboundDiscordInfo,
    ) -> Result<User, RegistrationError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let new_user = NewUser {
            username: format!("{}@discord", inbound.username),
            password: None,
        };

        let user = diesel::insert_into(user::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .map_err(|e| RegistrationError::InternalError(e.into()))?;

        let new_discord_info = NewDiscordInfo::from_inbound(inbound, user.id)?;

        diesel::insert_into(discord_info::table)
            .values(&new_discord_info)
            .execute(&mut conn)
            .map_err(|e| RegistrationError::InternalError(e.into()))?;

        Ok(user)
    }

    pub async fn link_account(
        &self,
        user: User,
        inbound: InboundDiscordInfo,
    ) -> Result<(), RegistrationError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let new_discord_info = NewDiscordInfo::from_inbound(inbound, user.id)?;

        diesel::insert_into(discord_info::table)
            .values(&new_discord_info)
            .execute(&mut conn)
            .map_err(|e| RegistrationError::InternalError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, inbound))]
    pub async fn register_user(&self, inbound: InboundUser) -> Result<User, RegistrationError> {
        tracing::info!(inbound.username, "registering user");

        let new_user: NewUser = inbound.try_into()?;

        // let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
        //     new_user
        //         .ed25519
        //         .as_slice()
        //         .try_into()
        //         .map_err(|_| AppError::InvalidKeySize)?,
        // )
        // .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        // let signature = Signature::from_bytes(
        //     new_user
        //         .signature
        //         .as_slice()
        //         .try_into()
        //         .map_err(|_| AppError::InvalidSignature)?,
        // );

        // let message = [
        //     new_user.username.as_bytes(),
        //     new_user.curve25519.as_slice(),
        //     new_user.ed25519.as_slice(),
        // ]
        // .concat();

        // verifying_key
        //     .verify_strict(&message, &signature)
        //     .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

        // let mut conn = self
        //     .pool
        //     .get()
        //     .map_err(|e| AppError::PoolError(e.to_string()))?;

        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = diesel::insert_into(user::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .map_err(|e| AppError::from(e))?;

        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    pub async fn new_device_for(&self, user: User) -> Result<Device, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let new_device = NewDevice {
            user_id: user.id,
            x25519: None,
            ed25519: None,
        };

        let device = diesel::insert_into(device::table)
            .values(&new_device)
            .returning(Device::as_returning())
            .get_result(&mut conn)
            .map_err(|e| AppError::from(e))?;

        Ok(device)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_device(&self, user: User, device_id: Uuid) -> Result<Device, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let device = device::table
            .filter(device::id.eq(device_id).and(device::user_id.eq(user.id)))
            .select(Device::as_select())
            .first(&mut conn)
            .map_err(|e| AppError::from(e))?;

        Ok(device)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_all_devices(&self, user: User) -> Result<Vec<Device>, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        device::table
            .filter(device::user_id.eq(user.id))
            .select(Device::as_select())
            .load(&mut conn)
            .map_err(|e| AppError::from(e))
    }

    pub async fn set_device_keys(
        &self,
        user: User,
        device_id: Uuid,
        device_keys: InboundDevice,
    ) -> Result<Device, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let new_device = NewDevice::from_network(device_id, device_keys)?;

        diesel::update(device::table)
            .filter(device::id.eq(device_id).and(device::user_id.eq(user.id)))
            .set((
                device::x25519.eq(new_device.x25519),
                device::ed25519.eq(new_device.ed25519),
            ))
            .get_result(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))
    }

    pub async fn get_otks(&self, device_id: Uuid) -> Result<Vec<Otk>, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        one_time_key::table
            .filter(one_time_key::device_id.eq(device_id))
            .select(Otk::as_select())
            .load(&mut conn)
            .map_err(|e| AppError::from(e))
    }

    pub async fn upload_otks(
        &self,
        user: User,
        device_id: Uuid,
        otks: InboundOtks,
    ) -> Result<(), AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;
        let signature = BASE64_STANDARD_NO_PAD.decode(&otks.signature)?;
        let signature = Signature::from_bytes(
            signature
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidSignature)?,
        );

        let device = device::table
            .filter(device::id.eq(device_id).and(device::user_id.eq(user.id)))
            .select(Device::as_select())
            .first(&mut conn)
            .map_err(|e| AppError::from(e))?;

        let otks: Vec<Curve25519PublicKey> = otks
            .otks
            .iter()
            .map(|k| {
                Curve25519PublicKey::from_base64(k).map_err(|e| AppError::InvalidKey(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let message = otks
            .iter()
            .map(|k| k.as_bytes() as &[u8])
            .collect::<Vec<&[u8]>>()
            .concat();

        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
            device
                .ed25519
                .ok_or(AppError::InvalidSignature)?
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidKeySize)?,
        )
        .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        verifying_key
            .verify_strict(&message, &signature)
            .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

        let new_otks = otks
            .into_iter()
            .map(|k| NewOtk {
                device_id,
                otk: k.to_bytes(),
            })
            .collect::<Vec<NewOtk>>();

        diesel::insert_into(one_time_key::table)
            .values(&new_otks)
            .execute(&mut conn)?;

        Ok(())
    }

    pub async fn change_nickname(&self, user: &User, nickname: &str) -> Result<(), AppError> {
        if !is_valid_nickname(nickname) {
            return Err(AppError::UserError("bad username".to_string()))
        }

        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        diesel::update(user::table.find(user.id))
            .set(user::nickname.eq(nickname.trim()))
            .execute(&mut conn)?;

        Ok(())
    }

    pub async fn get_known_users(&self, user: &User) -> Result<Vec<User>, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let sent_to_ids = channel::table
            .filter(channel::sender_id.eq(user.id))
            .select(channel::recipient_id);

        let received_from_ids = channel::table
            .filter(channel::recipient_id.eq(user.id))
            .select(channel::sender_id);

        let known_users = user::table
            .filter(
                user::id
                    .eq_any(sent_to_ids)
                    .or(user::id.eq_any(received_from_ids)),
            )
            .load::<User>(&mut conn)?;

        Ok(known_users)
    }

    async fn get_channel_participants(&self, channel_id: Uuid) -> Result<(User, User), AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let (sender, recipient) = alias!(
            crate::schema::user as sender,
            crate::schema::user as recipient
        );

        let (sender, recipient) = channel::table
            .find(channel_id)
            .inner_join(sender.on(channel::sender_id.eq(sender.field(user::id))))
            .inner_join(recipient.on(channel::recipient_id.eq(recipient.field(user::id))))
            .select((
                sender.fields(user::all_columns),
                recipient.fields(user::all_columns),
            ))
            .first::<(User, User)>(&mut conn)
            .map_err(|e| AppError::from(e))?;

        Ok((sender, recipient))
    }

    async fn get_other_channel_participant(
        &self,
        user: &User,
        channel_id: Uuid,
    ) -> Result<User, AppError> {
        let (sender, recipient) = self.get_channel_participants(channel_id).await?;

        Ok(if user.id == sender.id {
            recipient
        } else {
            sender
        })
    }

    pub async fn get_channel_info(
        &self,
        user: User,
        channel_id: Uuid,
    ) -> Result<ChannelInfo, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let other_user = self
            .get_other_channel_participant(&user, channel_id)
            .await?;

        let devices = device::table
            .filter(
                device::user_id
                    .eq(other_user.id)
                    .or(device::user_id.eq(user.id)),
            )
            .select(Device::as_select())
            .load(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?
            .into_iter()
            .map(|d| d.try_into())
            .collect::<Result<Vec<OutboundDevice>, _>>()?;

        Ok(ChannelInfo {
            channel_id,
            users: vec![user.into(), other_user.into()],
            devices,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_channels(&self, user: User) -> Result<Vec<ChannelResponse>, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let (sender, recipient) = alias!(
            crate::schema::user as sender,
            crate::schema::user as recipient
        );

        let channels = channel::table
            .inner_join(sender.on(channel::sender_id.eq(sender.field(user::id))))
            .inner_join(recipient.on(channel::recipient_id.eq(recipient.field(user::id))))
            .filter(
                channel::sender_id
                    .eq(user.id)
                    .or(channel::recipient_id.eq(user.id)),
            )
            .select((
                Channel::as_select(),
                sender.field(user::username),
                sender.field(user::nickname),
                recipient.field(user::username),
                recipient.field(user::nickname),
            ))
            .load::<(Channel, String, Option<String>, String, Option<String>)>(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        let channels = channels
            .into_iter()
            .map(|(channel, sender, sender_nick, receiver, receiver_nick)| {
                if channel.sender_id == user.id {
                    ChannelResponse {
                        channel_id: channel.id,
                        user_id: channel.recipient_id,
                        username: receiver,
                        nickname: receiver_nick,
                    }
                } else {
                    ChannelResponse {
                        channel_id: channel.id,
                        user_id: channel.sender_id,
                        username: sender,
                        nickname: sender_nick,
                    }
                }
            })
            .collect();

        Ok(channels)
    }

    pub async fn get_channel_history(
        &self,
        user: User,
        channel_id: Uuid,
        device_id: Uuid,
    ) -> Result<Vec<OutboundChatMessage>, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let history = message::table
            .inner_join(
                message_payload::table.on(message::id
                    .eq(message_payload::message_id)
                    .and(message_payload::recipient_device_id.eq(device_id))),
            )
            .inner_join(user::table.on(message::sender_id.eq(user::id)))
            .inner_join(device::table.on(message_payload::recipient_device_id.eq(device::id)))
            .filter(message::channel_id.eq(channel_id))
            .filter(device::user_id.eq(user.id))
            .select((
                message::id,
                message::sender_device_id,
                message::channel_id,
                message_payload::ciphertext,
                message::created,
                message_payload::is_pre_key,
            ))
            .order(message::id.asc())
            .load::<OutboundChatMessage>(&mut conn)
            .map_err(|e| AppError::from(e))?;

        Ok(history)
    }

    pub async fn create_channel_between(
        &self,
        sender: &User,
        recipient: &User,
    ) -> Result<(ChannelResponse, ChannelResponse), AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        if sender.id == recipient.id {
            return Err(AppError::UserError(
                "can't make chat with yourself".to_string(),
            ));
        }

        let new_channel = NewChannel {
            sender_id: sender.id,
            recipient_id: recipient.id,
        };

        let channel = diesel::insert_into(channel::table)
            .values(&new_channel)
            .returning(Channel::as_returning())
            .get_result(&mut conn)
            .map_err(|e| AppError::from(e))?;

        let (user1, user2) = self.get_channel_participants(channel.id).await?;

        let resp1 = ChannelResponse {
            channel_id: channel.id,
            user_id: user1.id,
            username: user1.username,
            nickname: user1.nickname,
        };
        let resp2 = ChannelResponse {
            channel_id: channel.id,
            user_id: user2.id,
            username: user2.username,
            nickname: user2.nickname,
        };

        if user1.id == sender.id {
            Ok((resp2, resp1))
        } else {
            Ok((resp1, resp2))
        }
    }

    pub async fn get_otk_for_device_in_channel(
        &self,
        user: User,
        channel_id: Uuid,
        device_id: Uuid,
    ) -> Result<Otk, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let other_user = self
            .get_other_channel_participant(&user, channel_id)
            .await?;

        let otk = one_time_key::table
            .inner_join(device::table.on(one_time_key::device_id.eq(device::id)))
            .filter(
                one_time_key::device_id
                    .eq(device_id)
                    .and(device::user_id.eq(other_user.id)),
            )
            .select(Otk::as_select())
            .first(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        diesel::delete(one_time_key::table.filter(one_time_key::id.eq(otk.id)))
            .execute(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(otk)
    }

    pub async fn save_message(
        &self,
        user: &User,
        message: InboundChatMessage,
    ) -> Result<(ChatMessage, Vec<MessagePayload>), AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let (user1, user2) = self.get_channel_participants(message.channel_id).await?;

        if !(user.id == user1.id || user.id == user2.id) {
            return Err(AppError::Unauthorized);
        }

        let new_message = NewChatMessage::from_inbound(&user, &message);
        let payloads = message
            .payloads
            .into_iter()
            .map(|m| m.to_new_message(message.message_id))
            .collect::<Result<Vec<NewMessagePayload>, _>>()?;

        let message = diesel::insert_into(message::table)
            .values(&new_message)
            .returning(ChatMessage::as_returning())
            .get_result(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        let payloads = diesel::insert_into(message_payload::table)
            .values(&payloads)
            .returning(MessagePayload::as_returning())
            .load(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok((message, payloads))
    }

    // pub async fn get_otk(&self, user: User) -> Result<Curve25519PublicKey, AppError> {
    //     let pool = self.pool.clone();
    //     let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //     let key = one_time_key::table
    //         .filter(one_time_key::user_id.eq(user.id))
    //         .select(Otk::as_select())
    //         .first(&mut conn)
    //         .map_err(|e| AppError::QueryFailed(e.to_string()))?;

    //     diesel::delete(one_time_key::dsl::one_time_key.filter(one_time_key::id.eq(key.id)))
    //         .execute(&mut conn)
    //         .map_err(|e| AppError::QueryFailed(e.to_string()))?;

    //     let key = Curve25519PublicKey::from_bytes(
    //         key.otk.try_into().map_err(|_| AppError::InvalidKeySize)?,
    //     );

    //     Ok(key)
    // }

    // pub async fn count_otks(&self, user: User) -> Result<i64, AppError> {
    //     let pool = self.pool.clone();
    //     let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //     let count = one_time_key::table
    //         .filter(one_time_key::user_id.eq(user.id))
    //         .count()
    //         .get_result(&mut conn)
    //         .map_err(|e| AppError::QueryFailed(e.to_string()))?;

    //     Ok(count)
    // }

    // pub async fn publish_otks(&self, user: User, otks: Vec<String>) -> Result<(), AppError> {
    //     let pool = self.pool.clone();
    //     let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //     let otks = otks
    //         .iter()
    //         .map(|key| -> Result<NewOtk, AppError> {
    //             Ok(NewOtk {
    //                 user_id: user.id,
    //                 otk: Curve25519PublicKey::from_base64(key)
    //                     .map_err(|e| AppError::InvalidKey(e.to_string()))?
    //                     .to_bytes(),
    //             })
    //         })
    //         .collect::<Result<Vec<NewOtk>, AppError>>()?;

    //     diesel::insert_into(one_time_key::table)
    //         .values(&otks)
    //         .execute(&mut conn)
    //         .map_err(|e| AppError::QueryFailed(e.to_string()))?;

    //     Ok(())
    // }

    // pub async fn get_other_channel_participant(
    //     &self,
    //     user: &User,
    //     channel_id: Uuid,
    // ) -> Result<User, AppError> {
    //     let pool = self.pool.clone();
    //     let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //     let other = channel::table
    //         .filter(channel::id.eq(channel_id))
    //         .inner_join(
    //             user::table.on(user::id
    //                 .eq(channel::sender)
    //                 .or(user::id.eq(channel::receiver))),
    //         )
    //         .filter(user::id.ne(user.id))
    //         .select(User::as_select())
    //         .first(&mut conn)
    //         .optional()
    //         .map_err(|e| AppError::QueryFailed(e.to_string()))?
    //         .ok_or(AppError::NoSuchUser)?;

    //     Ok(other)
    // }

    // pub async fn get_channel_broadcaster(
    //     &self,
    //     channel_id: Uuid,
    // ) -> broadcast::Sender<ChatMessage> {
    //     let mut channels = self.channels.write().await;
    //     let sender = channels
    //         .entry(channel_id)
    //         .or_insert(broadcast::Sender::new(128));
    //     sender.clone()
    // }

    // pub async fn save_message(&self, new_message: NewChatMessage) -> Result<ChatMessage, AppError> {
    //     let pool = self.pool.clone();
    //     let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //     let message = diesel::insert_into(message::table)
    //         .values(&new_message)
    //         .returning(ChatMessage::as_returning())
    //         .get_result(&mut conn)
    //         .map_err(|e| AppError::QueryFailed(e.to_string()))?;

    //     Ok(message)
    // }
}
