use crate::{ApiError, AppError, DeviceId, DeviceKeyService, InboundDevice, User};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};

#[tracing::instrument(skip(device_keys))]
pub async fn new_device(
    State(device_keys): State<impl DeviceKeyService>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    let new_device = device_keys.new_device_for(&user).await?;
    Ok(Json(new_device))
}

#[tracing::instrument(skip(device_keys))]
pub async fn upload_keys(
    State(device_keys): State<impl DeviceKeyService>,
    user: User,
    Path(device_id): Path<DeviceId>,
    Json(inbound_device_keys): Json<InboundDevice>,
) -> Result<impl IntoResponse, ApiError> {
    let device = device_keys
        .set_device_keys(&user, device_id, inbound_device_keys)
        .await?;
    Ok(Json(device))
}

#[tracing::instrument(skip(device_keys))]
pub async fn upload_keys_me(
    State(device_keys): State<impl DeviceKeyService>,
    user: User,
    Json(inbound_device_keys): Json<InboundDevice>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(device_id) = inbound_device_keys.device_id {
        let device = device_keys
            .set_device_keys(&user, device_id, inbound_device_keys)
            .await?;
        Ok(Json(device))
    } else {
        Err(AppError::UserError("no device id provided".to_string()).into())
    }
}
