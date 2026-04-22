use crate::{ApiError, AppError, AppState, Device, DeviceId, InboundDevice, User};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use vodozemac::{Curve25519PublicKey, Ed25519PublicKey};

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
    Path(device_id): Path<DeviceId>,
    Json(inbound_device_keys): Json<InboundDevice>,
) -> Result<impl IntoResponse, ApiError> {
    let device = set_device_keys(&app_state, user, device_id, inbound_device_keys).await?;
    Ok(Json(device))
}

#[tracing::instrument(skip(app_state))]
pub async fn upload_keys_me(
    State(app_state): State<AppState>,
    user: User,
    Json(inbound_device_keys): Json<InboundDevice>,
) -> Result<impl IntoResponse, ApiError> {
    let device_id = inbound_device_keys
        .device_id
        .ok_or_else(|| AppError::UserError("no device id provided".to_string()))?;
    let device = set_device_keys(&app_state, user, device_id, inbound_device_keys).await?;
    Ok(Json(device))
}

async fn set_device_keys(
    app_state: &AppState,
    user: User,
    device_id: DeviceId,
    inbound_device_keys: InboundDevice,
) -> Result<Device, AppError> {
    let existing: Vec<_> = app_state
        .device_keys
        .get_all_devices(&user)
        .await?
        .into_iter()
        .filter(|d| d.ed25519.is_some())
        .collect();

    if !existing.is_empty() {
        let authorization = inbound_device_keys
            .authorization
            .as_ref()
            .ok_or(AppError::MissingSignatureForDevice)?;

        let authorizing = existing
            .iter()
            .find(|d| d.id == authorization.authorizing_device_id)
            .ok_or_else(|| {
                AppError::UserError(
                    "authorizing_device_id is not a registered device for this user".into(),
                )
            })?;

        let authorizing_ed25519_bytes = authorizing
            .ed25519
            .as_deref()
            .ok_or_else(|| AppError::UserError("authorizing device has no ed25519 key".into()))?;
        let authorizing_ed25519_bytes: [u8; 32] = authorizing_ed25519_bytes
            .try_into()
            .map_err(|_| AppError::InvalidKey("stored authorizing ed25519 not 32 bytes".into()))?;
        let authorizing_key = Ed25519PublicKey::from_slice(&authorizing_ed25519_bytes)
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        let new_x25519 = Curve25519PublicKey::from_base64(&inbound_device_keys.x25519)
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;
        let new_ed25519 = Ed25519PublicKey::from_base64(&inbound_device_keys.ed25519)
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        authorization.verify(&authorizing_key, device_id, &new_x25519, &new_ed25519)?;
    }

    let device = app_state
        .device_keys
        .set_device_keys(&user, device_id, inbound_device_keys)
        .await?;

    Ok(device)
}
