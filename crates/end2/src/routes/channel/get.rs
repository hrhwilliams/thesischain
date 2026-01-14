use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{ApiError, AppState, User};

pub async fn get_channel_info(
    State(app_state): State<AppState>,
    user: User,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(app_state.get_channel_info(&user, channel_id).await?))
}

pub async fn get_all_channels(
    State(app_state): State<AppState>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(app_state.get_user_channels(&user).await?))
}
