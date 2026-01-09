use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{ApiError, AppState, InboundDevice, User};

#[tracing::instrument(skip(app_state))]
pub async fn new_device(
    State(app_state): State<AppState>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    let new_device = app_state.new_device_for(user).await?;
    Ok(Json(new_device))
}

#[tracing::instrument(skip(app_state))]
pub async fn get_device(
    State(app_state): State<AppState>,
    user: User,
    Path(device_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let device = app_state.get_device(user, device_id).await?;
    Ok(Json(device))
}

#[tracing::instrument(skip(app_state))]
pub async fn get_devices(
    State(app_state): State<AppState>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    let devices = app_state.get_all_devices(user).await?;
    Ok(Json(devices))
}

#[tracing::instrument(skip(app_state))]
pub async fn upload_keys(
    State(app_state): State<AppState>,
    user: User,
    Path(device_id): Path<Uuid>,
    Json(device_keys): Json<InboundDevice>,
) -> Result<impl IntoResponse, ApiError> {
    let device = app_state
        .set_device_keys(user, device_id, device_keys)
        .await?;
    Ok(Json(device))
}
