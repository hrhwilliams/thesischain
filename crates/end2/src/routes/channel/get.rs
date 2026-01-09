use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{ApiError, AppState, User};

#[derive(Deserialize)]
pub struct HistoryRequest {
    pub device_id: Uuid,
}

pub async fn get_channel_info(
    State(app_state): State<AppState>,
    user: User,
    Path(channel_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(app_state.get_channel_info(user, channel_id).await?))
}

pub async fn get_all_channels(
    State(app_state): State<AppState>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(app_state.get_user_channels(user).await?))
}

pub async fn get_channel_history(
    State(app_state): State<AppState>,
    user: User,
    Path(channel_id): Path<Uuid>,
    Json(HistoryRequest { device_id }): Json<HistoryRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(
        app_state
            .get_channel_history(user, channel_id, device_id)
            .await?,
    ))
}

pub async fn get_user_device_otk(
    State(app_state): State<AppState>,
    user: User,
    Path((channel_id, device_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(
        app_state
            .get_otk_for_device_in_channel(user, channel_id, device_id)
            .await?,
    ))
}
