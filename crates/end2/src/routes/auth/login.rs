use axum::{Json, extract::State, response::IntoResponse};
use secrecy::SecretString;
use serde::Deserialize;

use crate::{ApiError, AuthService, WebSession, WebSessionService};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: SecretString,
}

#[tracing::instrument(skip(auth, sessions, password))]
pub async fn login(
    State(auth): State<impl AuthService>,
    State(sessions): State<impl WebSessionService>,
    web_session: WebSession,
    Json(LoginRequest { username, password }): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user = auth.login(&username, password).await?;
    sessions
        .insert_into_session(web_session, "user_id".to_string(), user.id)
        .await?;
    Ok(Json(user))
}
