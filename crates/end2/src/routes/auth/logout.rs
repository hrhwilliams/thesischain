use crate::{ApiError, AppState, User, UserId, WebSession};
use axum::{Json, extract::State, response::IntoResponse};

#[tracing::instrument(skip(app_state))]
pub async fn logout(
    State(app_state): State<AppState>,
    web_session: WebSession,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    app_state
        .remove_from_session::<UserId>(web_session, "user_id")
        .await?;

    Ok(Json(serde_json::json!({ "status": "success" })))
}
