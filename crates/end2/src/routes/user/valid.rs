use axum::{Json, extract::State, response::IntoResponse};

use crate::{ApiError, AppState};

#[tracing::instrument(skip(app_state))]
pub async fn get_valid_users(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let count = app_state.device_keys.get_valid_users().await?;
    Ok(Json(serde_json::json!({ "valid_users": count })))
}
