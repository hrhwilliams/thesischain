use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use std::str::FromStr;
use uuid::Uuid;

use crate::{AppState, User};

pub enum ExtractError {
    NoSession,
    CookieError(String),
    InvalidSessionId(String),
    LookupError(String),
}

impl From<ExtractError> for Response {
    fn from(value: ExtractError) -> Self {
        match value {
            ExtractError::NoSession => StatusCode::UNAUTHORIZED.into_response(),
            ExtractError::CookieError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s).into_response(),
            ExtractError::InvalidSessionId(s) => (StatusCode::BAD_REQUEST, s).into_response(),
            ExtractError::LookupError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s).into_response(),
        }
    }
}

impl IntoResponse for ExtractError {
    fn into_response(self) -> Response {
        self.into()
    }
}

impl FromRequestParts<AppState> for User {
    type Rejection = ExtractError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ExtractError> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|e| ExtractError::CookieError(e.to_string()))?;

        if let Some(session_cookie) = jar.get("Session") {
            let session_id = Uuid::from_str(session_cookie.value())
                .map_err(|e| ExtractError::InvalidSessionId(e.to_string()))?;

            let user = state
                .get_user_from_session(session_id)
                .await
                .map_err(|e| ExtractError::LookupError(e.to_string()))?
                .ok_or(ExtractError::NoSession)?;

            Ok(user)
        } else {
            Err(ExtractError::NoSession)
        }
    }
}

impl OptionalFromRequestParts<AppState> for User {
    type Rejection = ExtractError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, ExtractError> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|e| ExtractError::CookieError(e.to_string()))?;

        if let Some(session_cookie) = jar.get("Session") {
            let session_id = Uuid::from_str(session_cookie.value())
                .map_err(|e| ExtractError::InvalidSessionId(e.to_string()))?;

            let user = state
                .get_user_from_session(session_id)
                .await
                .map_err(|e| ExtractError::LookupError(e.to_string()))?;

            Ok(user)
        } else {
            Ok(None)
        }
    }
}
