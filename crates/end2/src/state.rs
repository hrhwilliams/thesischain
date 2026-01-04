use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl, SelectableHelper, r2d2::ConnectionManager,
};
use diesel::{JoinOnDsl, alias};
use ed25519_dalek::Signature;
use r2d2::Pool;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;
use vodozemac::Curve25519PublicKey;

use crate::{
    AppError, Challenge, Channel, ChannelResponse, ChatMessage, KeyResponse, NewChallenge,
    NewChannel, NewChatMessage, NewOtk, NewSession, NewUser, NewUserB64, Otk, RegistrationError,
    Session, User, challenge, channel, message, one_time_key, session, user,
};

#[derive(Clone)]
pub struct AppState {
    pool: Pool<ConnectionManager<PgConnection>>,
    channels: Arc<RwLock<HashMap<Uuid, broadcast::Sender<ChatMessage>>>>,
}

impl AppState {
    #[must_use]
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self {
            pool,
            channels: Arc::default(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_by_username(&self, username: String) -> Result<Option<User>, AppError> {
        tracing::info!("querying for user");
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = user::table
            .filter(user::username.eq(username))
            .select(User::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(user)
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_session_for_user(&self, user: User) -> Result<Session, AppError> {
        tracing::info!("creating session for user");
        let new_session = NewSession { user_id: user.id };
        let pool = self.pool.clone();

        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let session = diesel::insert_into(session::table)
            .values(&new_session)
            .returning(Session::as_returning())
            .get_result::<Session>(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(session)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_from_session(&self, session_id: Uuid) -> Result<Option<User>, AppError> {
        tracing::info!("getting user from session token");
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = user::table
            .inner_join(session::table)
            .filter(session::id.eq(session_id))
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(user)
    }

    pub async fn remove_active_sessions(&self, user: User) -> Result<(), AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        diesel::delete(session::dsl::session.filter(session::user_id.eq(user.id)))
            .execute(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn register_user(&self, new_user: NewUserB64) -> Result<User, RegistrationError> {
        tracing::info!("registering user");
        if new_user.username.trim().is_empty() {
            return Err(RegistrationError::InvalidUsername);
        }

        let new_user: NewUser = new_user.try_into()?;

        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
            new_user
                .ed25519
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidKeySize)?,
        )
        .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        let signature = Signature::from_bytes(
            new_user
                .signature
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidSignature)?,
        );

        let message = [
            new_user.username.as_bytes(),
            new_user.curve25519.as_slice(),
            new_user.ed25519.as_slice(),
        ]
        .concat();

        verifying_key
            .verify_strict(&message, &signature)
            .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

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
    pub async fn generate_challenge_for(&self, username: String) -> Result<Challenge, AppError> {
        tracing::info!("generating challenge");
        let user = self
            .get_user_by_username(username)
            .await?
            .ok_or(AppError::NoSuchUser)?;

        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let new_challenge = NewChallenge { user_id: user.id };

        let challenge = diesel::insert_into(challenge::table)
            .values(&new_challenge)
            .returning(Challenge::as_returning())
            .get_result(&mut conn)
            .map_err(|e| AppError::from(e))?;

        Ok(challenge)
    }

    #[tracing::instrument(skip(self))]
    pub async fn verify_response_and_create_session(
        &self,
        id: Uuid,
        signature: String,
    ) -> Result<Session, AppError> {
        tracing::info!("verifying challenge response");
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = user::table
            .inner_join(challenge::table)
            .filter(challenge::id.eq(id))
            .select(User::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::from(e))?
            .ok_or(AppError::NoSuchUser)?;

        let message = [id.into_bytes(), user.id.into_bytes()].concat();

        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
            user.ed25519
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidKeySize)?,
        )
        .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        let signature = BASE64_STANDARD_NO_PAD.decode(signature)?;
        let signature = Signature::from_bytes(
            signature
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidSignature)?,
        );
        verifying_key
            .verify_strict(&message, &signature)
            .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

        self.create_session_for_user(user).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_channels(&self, user: User) -> Result<Vec<ChannelResponse>, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let (sender, receiver) = alias!(
            crate::schema::user as sender,
            crate::schema::user as receiver
        );

        let channels = channel::table
            .inner_join(sender.on(channel::sender.eq(sender.field(crate::schema::user::id))))
            .inner_join(receiver.on(channel::receiver.eq(receiver.field(crate::schema::user::id))))
            .filter(
                channel::sender
                    .eq(user.id)
                    .or(channel::receiver.eq(user.id)),
            )
            .select((
                Channel::as_select(),
                sender.field(user::username),
                receiver.field(user::username),
            ))
            .load::<(Channel, String, String)>(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        let channels = channels
            .into_iter()
            .map(|(channel, sender, receiver)| ChannelResponse {
                id: channel.id,
                sender,
                receiver,
            })
            .collect();

        tracing::info!("{:?}", channels);

        Ok(channels)
    }

    pub async fn create_channel_between(
        &self,
        sender: User,
        receiver: User,
    ) -> Result<Channel, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let new_channel = NewChannel {
            sender: sender.id,
            receiver: receiver.id,
        };

        let channel = diesel::insert_into(channel::table)
            .values(&new_channel)
            .returning(Channel::as_returning())
            .get_result(&mut conn)
            .map_err(|e| AppError::from(e))?;

        Ok(channel)
    }

    pub async fn get_identity_key(&self, user: User) -> KeyResponse {
        KeyResponse {
            kind: "id".to_string(),
            key: BASE64_STANDARD_NO_PAD.encode(user.curve25519),
        }
    }

    pub async fn get_otk(&self, user: User) -> Result<KeyResponse, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let key = one_time_key::table
            .filter(one_time_key::user_id.eq(user.id))
            .select(Otk::as_select())
            .first(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        diesel::delete(one_time_key::dsl::one_time_key.filter(one_time_key::id.eq(key.id)))
            .execute(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(KeyResponse {
            kind: "otk".to_string(),
            key: BASE64_STANDARD_NO_PAD.encode(key.otk),
        })
    }

    pub async fn count_otks(&self, user: User) -> Result<i64, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let count = one_time_key::table
            .filter(one_time_key::user_id.eq(user.id))
            .count()
            .get_result(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(count)
    }

    pub async fn publish_otks(&self, user: User, otks: Vec<String>) -> Result<(), AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let otks = otks
            .iter()
            .map(|key| -> Result<NewOtk, AppError> {
                Ok(NewOtk {
                    user_id: user.id,
                    otk: Curve25519PublicKey::from_base64(key)
                        .map_err(|e| AppError::InvalidKey(e.to_string()))?
                        .to_bytes(),
                })
            })
            .collect::<Result<Vec<NewOtk>, AppError>>()?;

        diesel::insert_into(one_time_key::table)
            .values(&otks)
            .execute(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}
