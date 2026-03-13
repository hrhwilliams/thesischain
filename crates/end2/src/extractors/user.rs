use crate::{ApiError, AppState, ExtractError, SessionId, User, UserId};
use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use base64::{Engine, prelude::BASE64_STANDARD};
use secrecy::SecretString;

impl User {
    async fn get_user_from_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Self, ExtractError> {
        tracing::debug!("extracting user from request");

        let jar = CookieJar::from_request_parts(parts, app_state)
            .await
            .map_err(|e| ExtractError::CookieError(e.to_string()))?;

        if let Some(session_cookie) = jar.get("Session") {
            let session_id = SessionId::try_from(session_cookie.value())
                .map_err(|e| ExtractError::InvalidSessionId(e.to_string()))?;
            let session = app_state
                .get_session(session_id)
                .await
                .map_err(ExtractError::LookupError)?
                .ok_or(ExtractError::NoSession)?;

            let user_id = app_state
                .get_from_session::<UserId>(&session, "user_id")
                .await
                .map_err(ExtractError::LookupError)?
                .ok_or(ExtractError::NoUser)?;

            let user = app_state
                .auth
                .get_user_info(user_id)
                .await
                .map_err(ExtractError::LookupError)?
                .ok_or(ExtractError::NoSession)?;

            Ok(user)
        } else if let Some(auth) = parts.headers.get("authorization") {
            let (auth, b64) = auth
                .to_str()
                .map_err(|e| {
                    ExtractError::CookieError(format!("invalid authorization header: {e}"))
                })?
                .split_once(' ')
                .ok_or_else(|| {
                    ExtractError::CookieError("invalid authorization header".to_string())
                })?;

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

            let (username, password): (&str, &str) = decoded.split_once(':').ok_or_else(|| {
                ExtractError::CookieError("invalid authorization header".to_string())
            })?;

            let user = app_state
                .auth
                .login(username, SecretString::from(password))
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
            .map_err(Into::into)
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
