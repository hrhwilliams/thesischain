use crate::{
    ApiError, AppError, AppState, AuthService, DeviceId, DeviceKeyService, MessageRelayService,
    OtkService, User, UserId, WebSessionService,
};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};

#[tracing::instrument(skip(app_state))]
pub async fn get_device<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
    Path(device_id): Path<DeviceId>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let device = app_state.get_device(&user, device_id).await?;
    Ok(Json(device))
}

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(app_state))]
pub async fn get_user_device<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    _user: User,
    Path((user_id, device_id)): Path<(UserId, DeviceId)>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let target_user = app_state
        .get_user_info(user_id)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    let device = app_state.get_device(&target_user, device_id).await?;
    Ok(Json(device))
}

#[tracing::instrument(skip(app_state))]
pub async fn get_devices<A, D, O, R, W>(
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
    let devices = app_state.get_all_devices(&user).await?;
    Ok(Json(devices))
}

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(app_state))]
pub async fn get_user_devices<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    _user: User,
    Path(user_id): Path<UserId>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let target_user = app_state
        .get_user_info(user_id)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    let devices = app_state.get_all_devices(&target_user).await?;
    Ok(Json(devices))
}
