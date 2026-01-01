use std::{collections::HashMap, sync::Arc};

use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand_core::OsRng;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{Session, SessionId, UserInfo, UserName};

pub enum AppError {
    ArgonError,
    UserExists,
}

#[derive(Clone, Default)]
pub struct AppState {
    users: Arc<RwLock<HashMap<UserName, UserInfo>>>,
    sessions: Arc<RwLock<HashMap<SessionId, UserName>>>,
}

impl AppState {
    pub async fn get_session(&self, session_id: &SessionId) -> Option<Session> {
        let user = {
            let sessions = self.sessions.read().await;
            tracing::info!("{:?}", sessions.keys());
            sessions.get(session_id).cloned()?
        };

        let user_info = {
            let users = self.users.read().await;
            users.get(&user).cloned()?
        };

        let session = Session {
            session_id: session_id.clone(),
            user_info,
        };

        Some(session)
    }

    pub async fn register_user(
        &self,
        user: UserName,
        password: &str,
        confirm: &str,
    ) -> Result<bool, AppError> {
        if password != confirm {
            return Ok(false);
        }

        if user.len() < 2 || user.len() > 16 {
            return Ok(false);
        }

        let user_exists = {
            let users = self.users.read().await;
            users.get(&user).is_some()
        };

        if user_exists {
            return Ok(false);
        }

        let salt = SaltString::generate(&mut OsRng);

        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AppError::ArgonError)?;

        let user_info = UserInfo {
            id: Uuid::new_v4(),
            username: user.clone(),
            password: hash.serialize(),
        };

        let mut users = self.users.write().await;
        users.insert(user, user_info);

        Ok(true)
    }

    pub async fn create_session(&self, user: UserName, password: &str) -> Option<Session> {
        let success = self.check_password(&user, password).await?;

        if success {
            let session_id = SessionId::new();

            let user_info = {
                let users = self.users.read().await;
                users.get(&user).cloned()
            }
            .unwrap();

            {
                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), user);
            }

            let session = Session {
                session_id,
                user_info,
            };

            Some(session)
        } else {
            None
        }
    }

    async fn check_password(&self, user: &UserName, password: &str) -> Option<bool> {
        let hash = {
            let users = self.users.read().await;
            users.get(user).and_then(|user| Some(user.password.clone()))
        }?;

        Some(
            Argon2::default()
                .verify_password(password.as_bytes(), &hash.password_hash())
                .is_ok(),
        )
    }
}
