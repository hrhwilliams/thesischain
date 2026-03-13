use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};

use crate::{AppError, SessionId, WebSession};

/// How the backend sets up and manages web sessions
#[async_trait]
pub trait WebSessionService: Clone + Send + Sync {
    async fn new_session(&self) -> Result<WebSession, AppError>;
    async fn get_session(&self, session_id: SessionId) -> Result<Option<WebSession>, AppError>;
    async fn insert_into_session<T: Serialize>(
        &self,
        web_session: WebSession,
        key: String,
        value: T,
    ) -> Result<WebSession, AppError>;
    async fn get_from_session<T: DeserializeOwned>(
        &self,
        web_session: &WebSession,
        key: &str,
    ) -> Result<Option<T>, AppError>;
    async fn remove_from_session<T: DeserializeOwned>(
        &self,
        web_session: WebSession,
        key: &str,
    ) -> Result<Option<(T, WebSession)>, AppError>;
}
