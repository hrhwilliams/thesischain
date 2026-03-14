use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;

use crate::{
    ApiError, AppError, AppState, AuthService, DeviceKeyService, MessageRelayService, OtkService,
    User, WebSession, WebSessionService,
};

#[tracing::instrument(skip(app_state))]
pub async fn get_google_oauth_url<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    web_session: WebSession,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let (google_url, csrf_token, pkce_verifier) = app_state
        .get_oauth_handler("google")
        .ok_or(AppError::ValueError("no google OAuth handler".to_string()))?
        .generate_oauth_url()
        .map_err(AppError::from)?;
    let web_session = app_state
        .insert_into_session(
            web_session,
            "csrf_token".to_string(),
            csrf_token.into_secret(),
        )
        .await?;
    app_state
        .insert_into_session(
            web_session,
            "pkce_verifier".to_string(),
            pkce_verifier.into_secret(),
        )
        .await?;

    Ok(Redirect::to(&google_url))
}

#[derive(Deserialize)]
pub struct OAuthResponse {
    pub code: String,
    pub state: String,
}

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip(app_state))]
pub async fn google_redirect<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    web_session: WebSession,
    user: Option<User>,
    Query(OAuthResponse { code, state }): Query<OAuthResponse>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let (csrf_token, web_session) = app_state
        .remove_from_session(web_session, "csrf_token")
        .await?
        .ok_or_else(|| AppError::ValueError("missing value".to_string()))?;
    let (pkce_verifier, _web_session) = app_state
        .remove_from_session(web_session, "pkce_verifier")
        .await?
        .ok_or_else(|| AppError::ValueError("missing value".to_string()))?;
    let token = app_state
        .get_oauth_handler("google")
        .ok_or(AppError::ValueError("no google OAuth handler".to_string()))?
        .get_google_token(code, state, csrf_token, pkce_verifier)
        .await
        .map_err(AppError::from)?;
    let _google_info = app_state
        .get_oauth_handler("google")
        .ok_or(AppError::ValueError("no google OAuth handler".to_string()))?
        .get_google_info(&token)
        .await
        .map_err(AppError::from)?;

    Ok(Redirect::to("https://chat.fiatlux.dev"))
}
