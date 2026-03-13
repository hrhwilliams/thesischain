use crate::{ApiError, AppError, AuthService, DeviceId, DeviceKeyService, User, UserId};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};

#[tracing::instrument(skip(device_keys))]
pub async fn get_device(
    State(device_keys): State<impl DeviceKeyService>,
    user: User,
    Path(device_id): Path<DeviceId>,
) -> Result<impl IntoResponse, ApiError> {
    let device = device_keys.get_device(&user, device_id).await?;
    Ok(Json(device))
}

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(auth, device_keys))]
pub async fn get_user_device(
    State(auth): State<impl AuthService>,
    State(device_keys): State<impl DeviceKeyService>,
    _user: User,
    Path((user_id, device_id)): Path<(UserId, DeviceId)>,
) -> Result<impl IntoResponse, ApiError> {
    let target_user = auth
        .get_user_info(user_id)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    let device = device_keys.get_device(&target_user, device_id).await?;
    Ok(Json(device))
}

#[tracing::instrument(skip(device_keys))]
pub async fn get_devices(
    State(device_keys): State<impl DeviceKeyService>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    let devices = device_keys.get_all_devices(&user).await?;
    Ok(Json(devices))
}

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(auth, device_keys))]
pub async fn get_user_devices(
    State(auth): State<impl AuthService>,
    State(device_keys): State<impl DeviceKeyService>,
    _user: User,
    Path(user_id): Path<UserId>,
) -> Result<impl IntoResponse, ApiError> {
    let target_user = auth
        .get_user_info(user_id)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    let devices = device_keys.get_all_devices(&target_user).await?;
    Ok(Json(devices))
}
