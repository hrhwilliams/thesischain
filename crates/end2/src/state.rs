use std::{collections::HashMap, sync::Arc};

use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand_core::OsRng;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;

use crate::{DirectMessageLink, Room, RoomId, Session, SessionId, UserInfo, UserName};

pub enum AppError {
    ArgonError,
    UserExists,
    NoSuchUser,
}

#[derive(Clone, Default)]
pub struct AppState {
    users: Arc<RwLock<HashMap<UserName, UserInfo>>>,
    dms: Arc<RwLock<HashMap<UserName, Vec<DirectMessageLink>>>>,
    rooms: Arc<RwLock<HashMap<RoomId, Room>>>,
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

        {
            let mut users = self.users.write().await;
            users.insert(user.clone(), user_info);
        }

        {
            let mut dms = self.dms.write().await;
            dms.insert(user, vec![]);
        }

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

    pub async fn get_direct_messages(&self, session: &Session) -> Option<Vec<DirectMessageLink>> {
        let user_dms = {
            let dms = self.dms.read().await;
            dms.get(session.username()).cloned()
        }?;

        Some(user_dms)
    }

    pub async fn create_dm(&self, session: &Session, recipient: UserName) -> Result<(), AppError> {
        let user_exists = {
            let users = self.users.read().await;
            users.get(session.username()).is_some()
        };

        let recipient_exists = {
            let users = self.users.read().await;
            users.get(&recipient).is_some()
        };

        if user_exists && recipient_exists {
            let id = RoomId::new();
            let mut dms = self.dms.write().await;

            dms.get_mut(session.username())
                .unwrap()
                .push(DirectMessageLink {
                    id: id.clone(),
                    user: recipient.clone(),
                });
            dms.get_mut(&recipient).unwrap().push(DirectMessageLink {
                id,
                user: session.username().clone(),
            });
            Ok(())
        } else {
            Err(AppError::NoSuchUser)
        }
    }

    pub async fn get_room(&self, room_id: RoomId) -> Room {
        let room = {
            let mut rooms = self.rooms.write().await;
            rooms
                .entry(room_id)
                .or_insert_with(|| {
                    let (tx, _rx) = broadcast::channel(128);
                    Room {
                        history: Arc::new(RwLock::new(vec![])),
                        sender: tx,
                    }
                })
                .clone()
        };

        room
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
