use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use std::str::FromStr;
use uuid::Uuid;

use crate::{ApiError, AppState, ExtractError, User};

impl User {
    async fn get_user_from_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Self, ExtractError> {
        let jar = CookieJar::from_request_parts(parts, app_state)
            .await
            .map_err(|e| ExtractError::CookieError(e.to_string()))?;

        if let Some(session_cookie) = jar.get("Session") {
            let session_id = Uuid::from_str(session_cookie.value())
                .map_err(|e| ExtractError::InvalidSessionId(e.to_string()))?;

            let user = app_state
                .get_user_from_session(session_id)
                .await
                .map_err(|e| ExtractError::LookupError(e))?
                .ok_or(ExtractError::NoSession)?;

            Ok(user)
        } else {
            Err(ExtractError::NoSession)
        }
    }
}

impl FromRequestParts<AppState> for User {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, app_state: &AppState) -> Result<Self, ApiError> {
        Self::get_user_from_parts(parts, app_state)
            .await
            .map_err(|e| e.into())
    }
}

impl OptionalFromRequestParts<AppState> for User {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Option<Self>, ApiError> {
        match Self::get_user_from_parts(parts, app_state).await {
            Ok(user) => Ok(Some(user)),
            Err(ExtractError::NoSession) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
