use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::{AppState, UserInfo};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

pub enum SessionError {
    ExtractError,
    NoSessionInRequest,
    NoSessionInDatabase,
}

impl From<SessionError> for Response {
    fn from(value: SessionError) -> Self {
        match value {
            SessionError::ExtractError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            SessionError::NoSessionInRequest => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            SessionError::NoSessionInDatabase => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl IntoResponse for SessionError {
    fn into_response(self) -> axum::response::Response {
        self.into()
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub session_id: SessionId,
    pub user_info: UserInfo,
    // store: HashMap<String, Value>,
}

impl Session {
    pub fn new(user_info: UserInfo) -> Self {
        Self {
            session_id: SessionId::new(),
            user_info,
        }
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id.clone()
    }
}

impl OptionalFromRequestParts<AppState> for Session {
    type Rejection = SessionError;
    // pub fn get<T>(&self, key: &str) -> Result<Option<T>, serde_json::Error>
    // where
    //     T: DeserializeOwned,
    // {
    //     if let Some(value) = self.store.get(key) {
    //         Ok(Some(serde_json::from_value(value.clone())?))
    //     } else {
    //         Ok(None)
    //     }
    // }

    // pub async fn set<T>(&mut self, key: &str, value: T) -> Result<(), DatabaseError>
    // where
    //     T: Serialize,
    // {
    //     self.store.insert(
    //         key.to_string(),
    //         serde_json::to_value(value).map_err(DatabaseError::SerdeError)?,
    //     );
    //     self.db.update_session_store(&self.session, &self.store).await
    // }

    // pub async fn remove(&mut self, key: &str) -> Result<(), DatabaseError> {
    //     self.store.remove(key);
    //     self.db.update_session_store(&self.session, &self.store).await
    // }

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, SessionError> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| SessionError::ExtractError)?;

        if let Some(session_cookie) = jar.get("Session") {
            let session_id = SessionId(session_cookie.value().to_string());

            if let Some(session) = state.get_session(&session_id).await {
                Ok(Some(session))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }

        // let session_id = jar
        //     .get("__Host-Http-Session")
        //     .ok_or_else(|| SessionError::NoSessionInRequest)?
        //     .value()
        //     .to_string();

        // let session = state
        //     .get_session(&session_id)
        //     .ok_or_else(|| SessionError::NoSessionInDatabase)?;

        // Ok(Some(session))
    }
}
