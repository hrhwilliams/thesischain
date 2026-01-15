use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use base64::{Engine, prelude::BASE64_STANDARD};
use std::str::FromStr;
use uuid::Uuid;

use crate::{ApiError, AppState, ExtractError, User};

impl User {
    async fn get_user_from_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Self, ExtractError> {
        tracing::info!("cookie");

        let jar = CookieJar::from_request_parts(parts, app_state)
            .await
            .map_err(|e| ExtractError::CookieError(e.to_string()))?;

        if let Some(session_cookie) = jar.get("Session") {
            let session_id = Uuid::from_str(session_cookie.value())
                .map_err(|e| ExtractError::InvalidSessionId(e.to_string()))?;
            let session = app_state
                .get_session(session_id)
                .await
                .map_err(|e| ExtractError::LookupError(e))?
                .ok_or(ExtractError::NoSession)?;

            let user_id = app_state
                .get_from_session::<Uuid>(&session, "user_id")
                .await
                .map_err(|e| ExtractError::LookupError(e))?
                .ok_or(ExtractError::NoUser)?;

            let user = app_state
                .get_user_info(user_id)
                .await
                .map_err(|e| ExtractError::LookupError(e))?
                .ok_or(ExtractError::NoSession)?;

            Ok(user)
        } else if let Some(auth) = parts.headers.get("authorization") {
            let (auth, b64) = auth
                .to_str()
                .map_err(|e| {
                    ExtractError::CookieError(format!("invalid authorization header: {}", e))
                })?
                .split_once(' ')
                .ok_or(ExtractError::CookieError(
                    "invalid authorization header".to_string(),
                ))?;

            if auth != "Basic" {
                return Err(ExtractError::CookieError(
                    "expected basic authorization".to_string(),
                ));
            }

            let decoded = String::from_utf8(
                BASE64_STANDARD
                    .decode(b64)
                    .map_err(|e| ExtractError::CookieError(e.to_string()))?,
            )
            .map_err(|_| ExtractError::CookieError("invalid authorization header".to_string()))?;

            let (username, password): (&str, &str) = decoded.split_once(':').ok_or(
                ExtractError::CookieError("invalid authorization header".to_string()),
            )?;

            let user = app_state
                .login(username, password)
                .await
                .map_err(|_| ExtractError::NoUser)?;

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
            Err(ExtractError::NoSession | ExtractError::NoUser) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
