use axum::{Json, extract::State, response::IntoResponse};
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    ApiError, AppState, AuthService, DeviceKeyService, MessageRelayService, OtkService, WebSession,
    WebSessionService,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: SecretString,
}

#[tracing::instrument(skip(app_state, password))]
pub async fn login<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    web_session: WebSession,
    Json(LoginRequest { username, password }): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let user = app_state.login(&username, password).await?;
    app_state
        .insert_into_session(web_session, "user_id".to_string(), user.id)
        .await?;
    Ok(Json(user))
}
