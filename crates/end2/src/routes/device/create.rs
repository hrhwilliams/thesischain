use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{ApiError, AppError, AppState, InboundDevice, User};

#[tracing::instrument(skip(app_state))]
pub async fn new_device(
    State(app_state): State<AppState>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    let new_device = app_state.device_keys.new_device_for(&user).await?;
    Ok(Json(new_device))
}

#[tracing::instrument(skip(app_state))]
pub async fn upload_keys(
    State(app_state): State<AppState>,
    user: User,
    Path(device_id): Path<Uuid>,
    Json(device_keys): Json<InboundDevice>,
) -> Result<impl IntoResponse, ApiError> {
    let device = app_state
        .device_keys
        .set_device_keys(&user, device_id, device_keys)
        .await?;
    Ok(Json(device))
}

#[tracing::instrument(skip(app_state))]
pub async fn upload_keys_me(
    State(app_state): State<AppState>,
    user: User,
    Json(device_keys): Json<InboundDevice>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(device_id) = device_keys.device_id {
        let device = app_state
            .device_keys
            .set_device_keys(&user, device_id, device_keys)
            .await?;
        Ok(Json(device))
    } else {
        Err(AppError::UserError("no device id provided".to_string()).into())
    }
}
