use crate::{ApiError, AppState, ExtractError, SessionId, WebSession};
use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;

impl FromRequestParts<AppState> for WebSession {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, app_state: &AppState) -> Result<Self, ApiError> {
        if let Some(session) = parts.extensions.get::<Self>().cloned() {
            return Ok(session);
        }

        let jar = CookieJar::from_request_parts(parts, app_state)
            .await
            .map_err(|e| ExtractError::CookieError(e.to_string()))?;
        let session_cookie = jar.get("Session").ok_or(ExtractError::NoSession)?;
        let session_id = SessionId::try_from(session_cookie.value())
            .map_err(|e| ExtractError::InvalidSessionId(e.to_string()))?;
        let session = app_state
            .web_sessions
            .get_session(session_id)
            .await?
            .ok_or(ExtractError::NoSession)?;

        Ok(session)
    }
}

impl OptionalFromRequestParts<AppState> for WebSession {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Option<Self>, ApiError> {
        if let Some(session) = parts.extensions.get::<Self>().cloned() {
            return Ok(Some(session));
        }

        let jar = CookieJar::from_request_parts(parts, app_state)
            .await
            .map_err(|e| ExtractError::CookieError(e.to_string()))?;

        if let Some(session_cookie) = jar.get("Session") {
            let session_id = SessionId::try_from(session_cookie.value())
                .map_err(|e| ExtractError::InvalidSessionId(e.to_string()))?;
            let session = app_state
                .web_sessions
                .get_session(session_id)
                .await?
                .ok_or(ExtractError::NoSession)?;

            Ok(Some(session))
        } else {
            Ok(None)
        }
    }
}
