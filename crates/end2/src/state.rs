use std::collections::HashMap;
use std::num::ParseIntError;
use std::sync::Arc;

use argon2::password_hash::Encoding;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl, SelectableHelper, r2d2::ConnectionManager,
};
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
    AppError, Channel, ChannelInfo, ChannelParticipant, ChatMessage, Device, InboundChatMessage,
    InboundDevice, InboundDiscordInfo, InboundOtks, InboundUser, LoginError, MessagePayload,
    NewChatMessage, NewDevice, NewDiscordInfo, NewMessagePayload, NewOtk, NewUser, OAuthHandler,
    Otk, OutboundChatMessage, RegistrationError, WebSession, WsEvent, channel_participant,
    discord_info, is_valid_nickname, message, message_payload, one_time_key,
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
            .or_insert_with(|| broadcast::Sender::new(128));
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
        match broadcaster.send(event) {
            Ok(_) => {}
            Err(e) => tracing::error!("failed to notify user: {}", e),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn new_session(&self) -> Result<WebSession, AppError> {
        let mut conn = self
            .pool
            .clone()
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let session = tokio::task::spawn_blocking(move || {
            diesel::insert_into(web_session::table)
                .default_values()
                .returning(WebSession::as_returning())
                .get_result(&mut conn)
        })
        .await??;

        Ok(session)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_session(&self, session_id: Uuid) -> Result<Option<WebSession>, AppError> {
        let mut conn = self
            .pool
            .clone()
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let session = tokio::task::spawn_blocking(move || {
            web_session::table
                .find(session_id)
                .select(WebSession::as_select())
                .first(&mut conn)
                .optional()
        })
        .await??;

        Ok(session)
    }

    pub async fn insert_into_session<T: Serialize>(
        &self,
        web_session: WebSession,
        key: String,
        value: T,
    ) -> Result<WebSession, AppError> {
        let mut conn = self
            .pool
            .clone()
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

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

        let web_session = tokio::task::spawn_blocking(move || {
            diesel::update(web_session::table)
                .filter(web_session::id.eq(web_session.id))
                .set(web_session::blob.eq(blob))
                .get_result(&mut conn)
        })
        .await??;

        Ok(web_session)
    }

    pub async fn get_from_session<T: DeserializeOwned>(
        &self,
        web_session: &WebSession,
        key: &str,
    ) -> Result<Option<T>, AppError> {
        let mut conn = self
            .pool
            .clone()
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let web_session_id = web_session.id;

        let blob = tokio::task::spawn_blocking(move || {
            web_session::table
                .find(web_session_id)
                .select(web_session::blob)
                .get_result(&mut conn)
        })
        .await??;

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
        let mut conn = self
            .pool
            .clone()
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let (value, blob) = match web_session.blob {
            Value::Object(mut m) => {
                let value = m.remove(key).and_then(|v| serde_json::from_value(v).ok());
                (value, Value::Object(m))
            }
            _ => unreachable!("only blob should be stored in web_session table"),
        };

        if let Some(value) = value {
            let web_session = tokio::task::spawn_blocking(move || {
                diesel::update(web_session::table)
                    .filter(web_session::id.eq(web_session.id))
                    .set(web_session::blob.eq(blob))
                    .get_result(&mut conn)
            })
            .await??;

            Ok(Some((value, web_session)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_user_info(&self, user_id: Uuid) -> Result<Option<User>, AppError> {
        let mut conn = self
            .pool
            .clone()
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = tokio::task::spawn_blocking(move || {
            user::table
                .find(user_id)
                .select(User::as_select())
                .first(&mut conn)
                .optional()
        })
        .await??;

        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let mut conn = self
            .pool
            .clone()
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let username = username.to_string();
        let user = tokio::task::spawn_blocking(move || {
            user::table
                .filter(user::username.eq(username))
                .select(User::as_select())
                .first(&mut conn)
                .optional()
        })
        .await??;

        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_by_discord_id(&self, discord_id: i64) -> Result<Option<User>, AppError> {
        let mut conn = self
            .pool
            .clone()
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = tokio::task::spawn_blocking(move || {
            discord_info::table
                .inner_join(user::table)
                .filter(discord_info::discord_id.eq(discord_id))
                .select(User::as_select())
                .first(&mut conn)
                .optional()
        })
        .await??;

        Ok(user)
    }

    #[tracing::instrument(skip(self, password))]
    pub async fn login(&self, username: &str, password: &str) -> Result<User, LoginError> {
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

    #[tracing::instrument(skip(self, inbound))]
    pub async fn register_with_discord(
        &self,
        inbound: InboundDiscordInfo,
    ) -> Result<User, RegistrationError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

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

    #[tracing::instrument(skip(self, inbound))]
    pub async fn link_account(
        &self,
        user: &User,
        inbound: InboundDiscordInfo,
    ) -> Result<(), RegistrationError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let new_discord_info = NewDiscordInfo::from_inbound(inbound, user.id)?;

        diesel::insert_into(discord_info::table)
            .values(&new_discord_info)
            .execute(&mut conn)
            .map_err(|e| RegistrationError::InternalError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, inbound))]
    pub async fn register_user(&self, inbound: InboundUser) -> Result<User, RegistrationError> {
        let new_user: NewUser = inbound.try_into()?;

        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = diesel::insert_into(user::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .map_err(AppError::from)?;

        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    pub async fn new_device_for(&self, user: &User) -> Result<Device, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

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
    pub async fn get_device(&self, user: &User, device_id: Uuid) -> Result<Device, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;
        let user_id = user.id;

        tracing::debug!("querying for device");

        let device = tokio::task::spawn_blocking(move || {
            device::table
                .filter(device::id.eq(device_id).and(device::user_id.eq(user_id)))
                .select(Device::as_select())
                .first(&mut conn)
        })
        .await??;

        Ok(device)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_all_devices(&self, user: &User) -> Result<Vec<Device>, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        device::table
            .filter(device::user_id.eq(user.id))
            .select(Device::as_select())
            .load(&mut conn)
            .map_err(|e| AppError::from(e))
    }

    pub async fn set_device_keys(
        &self,
        user: &User,
        device_id: Uuid,
        device_keys: InboundDevice,
    ) -> Result<Device, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;
        let new_device = NewDevice::from_network(device_id, device_keys)?;
        let user_id = user.id;

        let device = tokio::task::spawn_blocking(move || {
            diesel::update(device::table)
                .filter(device::id.eq(device_id).and(device::user_id.eq(user_id)))
                .set((
                    device::x25519.eq(new_device.x25519),
                    device::ed25519.eq(new_device.ed25519),
                ))
                .get_result(&mut conn)
        })
        .await??;

        Ok(device)
    }

    pub async fn get_otks(&self, device_id: Uuid) -> Result<Vec<Otk>, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let otks = tokio::task::spawn_blocking(move || {
            one_time_key::table
                .filter(one_time_key::device_id.eq(device_id))
                .select(Otk::as_select())
                .load(&mut conn)
        })
        .await??;

        Ok(otks)
    }

    #[tracing::instrument(skip(self, otks))]
    pub async fn upload_otks(
        &self,
        user: &User,
        device_id: Uuid,
        otks: InboundOtks,
    ) -> Result<(), AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let created_signature = BASE64_STANDARD_NO_PAD.decode(&otks.created_signature)?;
        let created_signature = Signature::from_bytes(
            created_signature
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidSignature)?,
        );

        let device = device::table
            .filter(device::id.eq(device_id).and(device::user_id.eq(user.id)))
            .select(Device::as_select())
            .first(&mut conn)?;

        let created_otks: Vec<Curve25519PublicKey> = otks
            .created
            .iter()
            .map(|k| {
                Curve25519PublicKey::from_base64(k).map_err(|e| AppError::InvalidKey(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let message = created_otks
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
            .verify_strict(&message, &created_signature)
            .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

        let new_otks = created_otks
            .into_iter()
            .map(|k| NewOtk {
                device_id,
                otk: k.to_bytes(),
            })
            .collect::<Vec<NewOtk>>();

        diesel::insert_into(one_time_key::table)
            .values(&new_otks)
            .execute(&mut conn)?;

        if let Some(removed_signature) = otks.removed_signature {
            tracing::info!("removing {} keys", otks.removed.len());

            let removed_signature = BASE64_STANDARD_NO_PAD.decode(&removed_signature)?;
            let removed_signature = Signature::from_bytes(
                removed_signature
                    .as_slice()
                    .try_into()
                    .map_err(|_| AppError::InvalidSignature)?,
            );

            let removed_otks: Vec<Curve25519PublicKey> = otks
                .removed
                .iter()
                .map(|k| {
                    Curve25519PublicKey::from_base64(k)
                        .map_err(|e| AppError::InvalidKey(e.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let removed_otks = removed_otks
                .iter()
                .map(|k| k.as_bytes() as &[u8])
                .collect::<Vec<&[u8]>>();

            verifying_key
                .verify_strict(&removed_otks.concat(), &removed_signature)
                .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

            diesel::delete(one_time_key::table)
                .filter(one_time_key::otk.eq_any(removed_otks))
                .execute(&mut conn)?;
        }

        Ok(())
    }

    pub async fn change_nickname(&self, user: &User, nickname: &str) -> Result<(), AppError> {
        if !is_valid_nickname(nickname) {
            return Err(AppError::UserError("bad username".to_string()));
        }

        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let user_id = user.id;
        let nickname = nickname.to_string();
        tokio::task::spawn_blocking(move || {
            diesel::update(user::table.find(user_id))
                .set(user::nickname.eq(nickname.trim()))
                .execute(&mut conn)
        })
        .await??;

        Ok(())
    }

    pub async fn get_known_users(&self, user: &User) -> Result<Vec<User>, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let user_id = user.id;
        let users = tokio::task::spawn_blocking(move || {
            let channel_ids = channel_participant::table
                .filter(channel_participant::user_id.eq(user_id))
                .select(channel_participant::channel_id)
                .load::<Uuid>(&mut conn)?;

            channel_participant::table
                .inner_join(user::table)
                .filter(channel_participant::channel_id.eq_any(channel_ids))
                .distinct()
                .select(User::as_select())
                .load(&mut conn)
        })
        .await??;

        Ok(users)
    }

    async fn get_channel_participants(&self, channel_id: Uuid) -> Result<Vec<User>, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

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

    #[tracing::instrument(skip(self))]
    pub async fn get_channel_info(
        &self,
        user: &User,
        channel_id: Uuid,
    ) -> Result<ChannelInfo, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

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
    pub async fn get_user_channels(&self, user: &User) -> Result<Vec<Channel>, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

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

    pub async fn get_channel_history(
        &self,
        user: &User,
        channel_id: Uuid,
        device_id: Uuid,
        after: Option<Uuid>,
    ) -> Result<Vec<OutboundChatMessage>, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

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
            query = query.filter(message::id.gt(after))
        }

        let history = query.load::<OutboundChatMessage>(&mut conn)?;
        Ok(history)
    }

    pub async fn create_channel_between(
        &self,
        sender: &User,
        recipient: &User,
    ) -> Result<ChannelInfo, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

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

    pub async fn get_user_otk(&self, user: &User, device_id: Uuid) -> Result<Otk, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let otk = one_time_key::table
            .inner_join(device::table.on(one_time_key::device_id.eq(device::id)))
            .filter(
                one_time_key::device_id
                    .eq(device_id)
                    .and(device::user_id.eq(user.id)),
            )
            .select(Otk::as_select())
            .first(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        diesel::delete(one_time_key::table.find(otk.id))
            .execute(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(otk)
    }

    pub async fn save_message(
        &self,
        user: &User,
        message: InboundChatMessage,
    ) -> Result<(ChatMessage, Vec<MessagePayload>), AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let users = self.get_channel_participants(message.channel_id).await?;

        if !users.contains(user) {
            return Err(AppError::Unauthorized);
        }

        let new_message = NewChatMessage::from_inbound(user, &message);
        let payloads = message
            .payloads
            .into_iter()
            .map(|m| m.to_new_message(message.message_id))
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
}
