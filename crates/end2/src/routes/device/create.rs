use crate::{
    ApiError, AppError, AppState, AuthService, DeviceId, DeviceKeyService, InboundDevice,
    MessageRelayService, OtkService, User, WebSessionService,
};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};

#[tracing::instrument(skip(app_state))]
pub async fn new_device<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let new_device = app_state.new_device_for(&user).await?;
    Ok(Json(new_device))
}

#[tracing::instrument(skip(app_state))]
pub async fn upload_keys<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
    Path(device_id): Path<DeviceId>,
    Json(inbound_device_keys): Json<InboundDevice>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let device = app_state
        .set_device_keys(&user, device_id, inbound_device_keys)
        .await?;
    Ok(Json(device))
}

#[tracing::instrument(skip(app_state))]
pub async fn upload_keys_me<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
    Json(inbound_device_keys): Json<InboundDevice>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    if let Some(device_id) = inbound_device_keys.device_id {
        let device = app_state
            .set_device_keys(&user, device_id, inbound_device_keys)
            .await?;
        Ok(Json(device))
    } else {
        Err(AppError::UserError("no device id provided".to_string()).into())
    }
}
