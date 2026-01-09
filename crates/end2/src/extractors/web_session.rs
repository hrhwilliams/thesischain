use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use std::str::FromStr;
use uuid::Uuid;

use crate::{ApiError, AppState, ExtractError, WebSession};

impl FromRequestParts<AppState> for WebSession {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, app_state: &AppState) -> Result<Self, ApiError> {
        let jar = CookieJar::from_request_parts(parts, app_state)
            .await
            .map_err(|e| ExtractError::CookieError(e.to_string()))?;
        let session_cookie = jar.get("Session").ok_or(ExtractError::NoSession)?;
        let session_id = Uuid::from_str(session_cookie.value())
            .map_err(|e| ExtractError::InvalidSessionId(e.to_string()))?;
        let session = app_state
            .get_session(session_id)
            .await?
            .ok_or(ExtractError::NoSession)?;

        Ok(session)
    }
}
