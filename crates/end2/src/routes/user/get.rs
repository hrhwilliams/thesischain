use crate::{ApiError, AppState, UserId};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};

#[tracing::instrument(skip(app_state))]
pub async fn get_user_info(
    State(app_state): State<AppState>,
    Path(user_id): Path<UserId>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(app_state.auth.get_user_info(user_id).await?))
}
