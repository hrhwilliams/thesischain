use std::collections::HashMap;
use std::sync::Arc;

use argon2::password_hash::Encoding;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, OptionalExtension, PgConnection,
    QueryDsl, RunQueryDsl, SelectableHelper, r2d2::ConnectionManager,
};
use ed25519_dalek::Signature;
use r2d2::Pool;
use rand_core::{OsRng, RngCore};
use tokio::sync::broadcast::Sender;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::{
    AppError, Challenge, ChatMessage, NewChallenge, NewChatMessage, NewSession, NewUser,
    RegistrationError, Room, Session, User, challenge, messages, room_participants, rooms,
    sessions, users,
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

    // pub async fn validate_password_and_get_user(
    //     &self,
    //     username: String,
    //     password: String,
    // ) -> Result<User, LoginError> {
    //     let pool = self.pool.clone();

    //     let user = tokio::task::spawn_blocking(move || {
    //         let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //         users::dsl::users
    //             .filter(users::columns::username.eq(username))
    //             .first::<User>(&mut conn)
    //             .optional()
    //             .map_err(|e| AppError::QueryFailed(e.to_string()))
    //     })
    //     .await
    //     .map_err(|e| AppError::JoinError(e.to_string()))??
    //     .ok_or(LoginError::UserNotFound)?;

    //     let password_hash = PasswordHash::parse(&user.pass, Encoding::B64)
    //         .map_err(|e| AppError::ArgonError(e.to_string()))?;

    //     let is_correct = Argon2::default()
    //         .verify_password(password.as_bytes(), &password_hash)
    //         .is_ok();

    //     if is_correct {
    //         Ok(user)
    //     } else {
    //         Err(LoginError::InvalidPassword)
    //     }
    // }

    pub async fn get_user_by_username(&self, username: String) -> Result<Option<User>, AppError> {
        let pool = self.pool.clone();
        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = users::table
            .filter(users::username.eq(username))
            .select(User::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(user)
    }

    pub async fn create_session_for_user(&self, user: User) -> Result<Session, AppError> {
        let new_session = NewSession { user_id: user.id };
        let pool = self.pool.clone();

        let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

        let session = diesel::insert_into(sessions::table)
            .values(&new_session)
            .returning(Session::as_returning())
            .get_result::<Session>(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(session)
    }

    // pub async fn get_user_from_session(&self, session_id: Uuid) -> Result<Option<User>, AppError> {
    //     let pool = self.pool.clone();

    //     tokio::task::spawn_blocking(move || {
    //         let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //         users::table
    //             .inner_join(sessions::table)
    //             .filter(sessions::id.eq(session_id))
    //             .select(User::as_select())
    //             .first::<User>(&mut conn)
    //             .optional()
    //             .map_err(|e| AppError::QueryFailed(e.to_string()))
    //     })
    //     .await
    //     .map_err(|e| AppError::JoinError(e.to_string()))?
    // }

    // pub async fn delete_user_session(&self, user: User) -> Result<(), AppError> {
    //     let pool = self.pool.clone();

    //     tokio::task::spawn_blocking(move || {
    //         let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //         diesel::delete(sessions::dsl::sessions.filter(sessions::user_id.eq(user.id)))
    //             .execute(&mut conn)
    //             .map_err(|e| AppError::QueryFailed(e.to_string()))
    //     })
    //     .await
    //     .map_err(|e| AppError::JoinError(e.to_string()))??;

    //     Ok(())
    // }

    pub async fn register_user(&self, new_user: NewUser) -> Result<User, RegistrationError> {
        if new_user.username.trim().is_empty() {
            return Err(RegistrationError::InvalidUsername);
        }

        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
            new_user.ed25519
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidKeySize)?,
        )
        .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        let signature = Signature::from_bytes(
            new_user.signature
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidSignature)?,
        );

        let message = [
            new_user.curve25519.as_slice(), 
            new_user.ed25519.as_slice()
        ].concat();

        verifying_key
            .verify_strict(&message, &signature)
            .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .map_err(|e| AppError::from(e))?;

        Ok(user)
    }

    pub async fn generate_challenge_for(&self, username: String) -> Result<Challenge, AppError> {
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

    pub async fn verify_response_and_create_session(
        &self,
        id: Uuid,
        signature: String,
    ) -> Result<Session, AppError> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))?;

        let user = users::table
            .inner_join(challenge::table)
            .filter(challenge::id.eq(id))
            .select(User::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| AppError::from(e))?
            .ok_or(AppError::NoSuchUser)?;

        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
            user.ed25519
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidKeySize)?,
        )
        .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        let signature = BASE64_URL_SAFE
            .decode(signature)
            .map_err(|_| AppError::InvalidB64)?;
        let signature = Signature::from_bytes(
            signature
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidSignature)?,
        );
        verifying_key
            .verify_strict(id.as_bytes(), &signature)
            .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

        self.create_session_for_user(user).await
    }

    // pub async fn get_rooms(&self, user: User) -> Result<Vec<Room>, AppError> {
    //     let pool = self.pool.clone();

    //     tokio::task::spawn_blocking(move || {
    //         let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //         rooms::table
    //             .inner_join(room_participants::table)
    //             .filter(room_participants::user_id.eq(user.id))
    //             .select(Room::as_select())
    //             .load(&mut conn)
    //             .map_err(|e| AppError::QueryFailed(e.to_string()))
    //     })
    //     .await
    //     .map_err(|e| AppError::JoinError(e.to_string()))?
    // }

    // pub async fn create_room(&self, user: User) -> Result<Room, AppError> {
    //     let pool = self.pool.clone();

    //     tokio::task::spawn_blocking(move || {
    //         let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //         conn.transaction::<Room, AppError, _>(|conn| {
    //             let room = diesel::insert_into(rooms::table)
    //                 .default_values()
    //                 .returning(Room::as_returning())
    //                 .get_result(conn)?;

    //             diesel::insert_into(room_participants::table)
    //                 .values((
    //                     room_participants::room_id.eq(room.id),
    //                     room_participants::user_id.eq(user.id),
    //                 ))
    //                 .execute(conn)?;

    //             Ok(room)
    //         })
    //     })
    //     .await
    //     .map_err(|e| AppError::JoinError(e.to_string()))?
    // }

    // pub async fn user_has_access(&self, user: &User, room_id: Uuid) -> Result<bool, AppError> {
    //     let pool = self.pool.clone();
    //     let user_id = user.id;

    //     tokio::task::spawn_blocking(move || {
    //         let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //         diesel::select(diesel::dsl::exists(
    //             room_participants::table.filter(
    //                 room_participants::room_id
    //                     .eq(room_id)
    //                     .and(room_participants::user_id.eq(user_id)),
    //             ),
    //         ))
    //         .get_result(&mut conn)
    //         .map_err(|e| AppError::QueryFailed(e.to_string()))
    //     })
    //     .await
    //     .map_err(|e| AppError::JoinError(e.to_string()))?
    // }

    // pub async fn invite_to_room(&self, user: User, room: &Room) -> Result<(), AppError> {
    //     let pool = self.pool.clone();
    //     let room_id = room.id;

    //     tokio::task::spawn_blocking(move || {
    //         let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //         diesel::insert_into(room_participants::table)
    //             .values((
    //                 room_participants::room_id.eq(room_id),
    //                 room_participants::user_id.eq(user.id),
    //             ))
    //             .execute(&mut conn)
    //             .map_err(|e| AppError::QueryFailed(e.to_string()))
    //     })
    //     .await
    //     .map_err(|e| AppError::JoinError(e.to_string()))??;

    //     Ok(())
    // }

    // pub async fn get_room_history(&self, room_id: Uuid) -> Result<Vec<ChatMessage>, AppError> {
    //     let pool = self.pool.clone();

    //     tokio::task::spawn_blocking(move || {
    //         let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //         messages::table
    //             .filter(messages::room_id.eq(room_id))
    //             .order(messages::id.asc())
    //             .select(ChatMessage::as_select())
    //             .load(&mut conn)
    //             .map_err(|e| AppError::QueryFailed(e.to_string()))
    //     })
    //     .await
    //     .map_err(|e| AppError::JoinError(e.to_string()))?
    // }

    // pub async fn get_channel(&self, room_id: Uuid) -> Sender<ChatMessage> {
    //     let mut channels = self.channels.write().await;
    //     channels
    //         .entry(room_id)
    //         .or_insert_with(|| {
    //             let (tx, _rx) = broadcast::channel(128);
    //             tx
    //         })
    //         .clone()
    // }

    // pub async fn save_message(&self, new_message: NewChatMessage) -> Result<ChatMessage, AppError> {
    //     let pool = self.pool.clone();

    //     tokio::task::spawn_blocking(move || {
    //         let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    //         diesel::insert_into(messages::table)
    //             .values(new_message)
    //             .returning(ChatMessage::as_returning())
    //             .get_result(&mut conn)
    //             .map_err(|e| AppError::QueryFailed(e.to_string()))
    //     })
    //     .await
    //     .map_err(|e| AppError::JoinError(e.to_string()))?
    // }

    // pub async fn get_direct_messages(&self, session: &Session) -> Option<Vec<DirectMessageLink>> {
    //     let user_dms = {
    //         let dms = self.dms.read().await;
    //         dms.get(session.username()).cloned()
    //     }?;

    //     Some(user_dms)
    // }

    // pub async fn create_dm(&self, session: &Session, recipient: UserName) -> Result<(), AppError> {
    //     let user_exists = {
    //         let users = self.users.read().await;
    //         users.get(session.username()).is_some()
    //     };

    //     let recipient_exists = {
    //         let users = self.users.read().await;
    //         users.get(&recipient).is_some()
    //     };

    //     if user_exists && recipient_exists {
    //         let id = RoomId::new();
    //         let mut dms = self.dms.write().await;

    //         dms.get_mut(session.username())
    //             .unwrap()
    //             .push(DirectMessageLink {
    //                 id: id.clone(),
    //                 user: recipient.clone(),
    //             });
    //         dms.get_mut(&recipient).unwrap().push(DirectMessageLink {
    //             id,
    //             user: session.username().clone(),
    //         });
    //         Ok(())
    //     } else {
    //         Err(AppError::NoSuchUser)
    //     }
    // }

    // pub async fn get_room(&self, room_id: RoomId) -> Room {
    //     let room = {
    //         let mut rooms = self.rooms.write().await;
    //         rooms
    //             .entry(room_id)
    //             .or_insert_with(|| {
    //                 let (tx, _rx) = broadcast::channel(128);
    //                 Room {
    //                     history: Arc::new(RwLock::new(vec![])),
    //                     sender: tx,
    //                 }
    //             })
    //             .clone()
    //     };

    //     room
    // }

    // async fn check_password(&self, user: &UserName, password: &str) -> Option<bool> {
    //     let hash = {
    //         let users = self.users.read().await;
    //         users.get(user).and_then(|user| Some(user.password.clone()))
    //     }?;

    //     Some(
    //         Argon2::default()
    //             .verify_password(password.as_bytes(), &hash.password_hash())
    //             .is_ok(),
    //     )
    // }
}
