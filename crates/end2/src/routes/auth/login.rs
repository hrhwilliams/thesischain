use axum::{Json, extract::State, response::IntoResponse};
use serde::Deserialize;

use crate::{ApiError, AppState, WebSession};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[tracing::instrument(skip(app_state, password))]
pub async fn login(
    State(app_state): State<AppState>,
    web_session: WebSession,
    Json(LoginRequest { username, password }): Json<LoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user = app_state.login(&username, &password).await?;
    app_state
        .insert_into_session(web_session, "user_id".to_string(), user.id)
        .await?;
    Ok(Json(user))
}
