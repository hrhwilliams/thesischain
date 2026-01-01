use std::str::FromStr;

use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::AppState;

pub enum SessionError {
    ExtractError,
    InvalidSessionId,
}

impl From<SessionError> for Response {
    fn from(value: SessionError) -> Self {
        match value {
            SessionError::ExtractError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            SessionError::InvalidSessionId => StatusCode::BAD_REQUEST.into_response(),
        }
    }
}

impl IntoResponse for SessionError {
    fn into_response(self) -> axum::response::Response {
        self.into()
    }
}

#[derive(Clone, Debug)]
pub struct SessionCookie(pub Uuid);

impl OptionalFromRequestParts<AppState> for SessionCookie {
    type Rejection = SessionError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, SessionError> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| SessionError::ExtractError)?;

        if let Some(session_cookie) = jar.get("Session") {
            let session_id = Uuid::from_str(session_cookie.value())
                .map_err(|_| SessionError::InvalidSessionId)?;

            Ok(Some(SessionCookie(session_id)))
        } else {
            Ok(None)
        }
    }
}
