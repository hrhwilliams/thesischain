use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{ApiError, AppError, AppState, User};

#[tracing::instrument(skip(app_state))]
pub async fn get_device(
    State(app_state): State<AppState>,
    user: User,
    Path(device_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let device = app_state.keys.get_device(&user, device_id).await?;
    Ok(Json(device))
}

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(app_state))]
pub async fn get_user_device(
    State(app_state): State<AppState>,
    _user: User,
    Path((user_id, device_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let target_user = app_state
        .auth
        .get_user_info(user_id)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    let device = app_state.keys.get_device(&target_user, device_id).await?;
    Ok(Json(device))
}

#[tracing::instrument(skip(app_state))]
pub async fn get_devices(
    State(app_state): State<AppState>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    let devices = app_state.keys.get_all_devices(&user).await?;
    Ok(Json(devices))
}

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(app_state))]
pub async fn get_user_devices(
    State(app_state): State<AppState>,
    _user: User,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let target_user = app_state
        .auth
        .get_user_info(user_id)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    let devices = app_state.keys.get_all_devices(&target_user).await?;
    Ok(Json(devices))
}
