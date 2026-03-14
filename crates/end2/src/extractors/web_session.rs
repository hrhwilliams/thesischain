use crate::{
    ApiError, AppState, AuthService, DeviceKeyService, ExtractError, MessageRelayService,
    OtkService, SessionId, WebSession, WebSessionService,
};
use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;

impl<
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService,
> FromRequestParts<AppState<A, D, O, R, W>> for WebSession
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState<A, D, O, R, W>,
    ) -> Result<Self, ApiError> {
        let jar = CookieJar::from_request_parts(parts, app_state)
            .await
            .map_err(|e| ExtractError::CookieError(e.to_string()))?;
        let session_cookie = jar.get("Session").ok_or(ExtractError::NoSession)?;
        let session_id = SessionId::try_from(session_cookie.value())
            .map_err(|e| ExtractError::InvalidSessionId(e.to_string()))?;
        let session = app_state
            .get_session(session_id)
            .await?
            .ok_or(ExtractError::NoSession)?;

        Ok(session)
    }
}

impl<
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService,
> OptionalFromRequestParts<AppState<A, D, O, R, W>> for WebSession
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState<A, D, O, R, W>,
    ) -> Result<Option<Self>, ApiError> {
        let jar = CookieJar::from_request_parts(parts, app_state)
            .await
            .map_err(|e| ExtractError::CookieError(e.to_string()))?;
        if let Some(session_cookie) = jar.get("Session") {
            let session_id = SessionId::try_from(session_cookie.value())
                .map_err(|e| ExtractError::InvalidSessionId(e.to_string()))?;
            let session = app_state
                .get_session(session_id)
                .await?
                .ok_or(ExtractError::NoSession)?;

            Ok(Some(session))
        } else {
            Ok(None)
        }
    }
}
