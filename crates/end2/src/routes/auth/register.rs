use axum::{Json, extract::State, response::IntoResponse};

use crate::{ApiError, AppState, InboundUser, WebSession};

/// POST with JSON payload `{ username: string, password: string, confirm_password: string }`
#[tracing::instrument(skip(app_state, new_user))]
pub async fn register(
    State(app_state): State<AppState>,
    web_session: WebSession,
    Json(new_user): Json<InboundUser>,
) -> Result<impl IntoResponse, ApiError> {
    let user = app_state.register_user(new_user).await?;
    app_state
        .insert_into_session(web_session, "user_id".to_string(), user.id)
        .await?;
    Ok(Json(user))
}
