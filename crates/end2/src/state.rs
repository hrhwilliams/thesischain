use argon2::password_hash::Encoding;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use diesel::{
    ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper,
    r2d2::ConnectionManager,
};
use r2d2::Pool;
use rand_core::OsRng;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::{NewSession, NewUser, Room, Session, User, room_participants, rooms, sessions, users};

#[derive(Debug)]
pub enum AppError {
    ArgonError(String),
    JoinError(String),
    QueryFailed(String),
    InsertFailed(String),
    PoolError(String),
}

pub enum LoginError {
    UserNotFound,
    InvalidPassword,
    System(AppError),
}

impl From<AppError> for LoginError {
    fn from(inner: AppError) -> Self {
        LoginError::System(inner)
    }
}

pub enum RegistrationError {
    InvalidUsername,
    InvalidPassword,
    PasswordMismatch,
    UsernameTaken,
    System(AppError),
}

// Helper to wrap system errors
impl From<AppError> for RegistrationError {
    fn from(inner: AppError) -> Self {
        RegistrationError::System(inner)
    }
}

impl AppError {
    pub fn to_string(&self) -> String {
        match self {
            Self::ArgonError(s) => s.clone(),
            Self::JoinError(s) => s.clone(),
            Self::QueryFailed(s) => s.clone(),
            Self::InsertFailed(s) => s.clone(),
            Self::PoolError(s) => s.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl AppState {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }

    pub async fn validate_password_and_get_user(
        &self,
        username: String,
        password: String,
    ) -> Result<User, LoginError> {
        let pool = self.pool.clone();

        let user = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

            users::dsl::users
                .filter(users::columns::username.eq(username))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| AppError::QueryFailed(e.to_string()))
        })
        .await
        .map_err(|e| AppError::JoinError(e.to_string()))??
        .ok_or(LoginError::UserNotFound)?;

        let password_hash = PasswordHash::parse(&user.pass, Encoding::B64)
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

    pub async fn create_session_for_user(&self, user: User) -> Result<Session, AppError> {
        let new_session = NewSession { user_id: user.id };
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

            diesel::insert_into(sessions::table)
                .values(&new_session)
                .returning(Session::as_returning())
                .get_result::<Session>(&mut conn)
                .map_err(|e| AppError::QueryFailed(e.to_string()))
        })
        .await
        .map_err(|e| AppError::JoinError(e.to_string()))?
    }

    pub async fn get_user_from_session(&self, session_id: Uuid) -> Result<Option<User>, AppError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

            users::table
                .inner_join(sessions::table)
                .filter(sessions::id.eq(session_id))
                .select(User::as_select())
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| AppError::QueryFailed(e.to_string()))
        })
        .await
        .map_err(|e| AppError::JoinError(e.to_string()))?
    }

    pub async fn delete_user_session(&self, user: User) -> Result<(), AppError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

            diesel::delete(sessions::dsl::sessions.filter(sessions::user_id.eq(user.id)))
                .execute(&mut conn)
                .map_err(|e| AppError::QueryFailed(e.to_string()))
        })
        .await
        .map_err(|e| AppError::JoinError(e.to_string()))??;

        Ok(())
    }

    pub async fn register_user(
        &self,
        username: String,
        password: String,
        confirmation: String,
    ) -> Result<User, RegistrationError> {
        if username.trim().is_empty() {
            return Err(RegistrationError::InvalidUsername);
        }
        if password.is_empty() {
            return Err(RegistrationError::InvalidPassword);
        }
        if password != confirmation {
            return Err(RegistrationError::PasswordMismatch);
        }

        let pool = self.pool.clone();

        let user = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            let hash = Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| AppError::ArgonError(e.to_string()))?
                .to_string();

            let new_user = NewUser {
                username: &username,
                pass: &hash,
            };

            let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

            diesel::insert_into(users::table)
                .values(&new_user)
                .returning(User::as_returning())
                .get_result(&mut conn)
                .map_err(|e| AppError::InsertFailed(e.to_string()))
        })
        .await
        .map_err(|e| AppError::JoinError(e.to_string()))??;

        Ok(user)
    }

    pub async fn get_rooms(&self, user: User) -> Result<Vec<Room>, AppError> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

            rooms::table
                .inner_join(room_participants::table)
                .filter(room_participants::user_id.eq(user.id))
                .select(Room::as_select())
                .load(&mut conn)
                .map_err(|e| AppError::QueryFailed(e.to_string()))
        })
        .await
        .map_err(|e| AppError::JoinError(e.to_string()))?
    }

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
