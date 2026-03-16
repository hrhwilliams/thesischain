use crate::{
    ApiError, AppError, AppState, AuthService, DeviceId, DeviceKeyService, InboundOtks,
    MessageRelayService, OtkService, User, UserId, WebSessionService,
};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(app_state))]
pub async fn get_otks<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    _user: User,
    Path(device_id): Path<DeviceId>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    Ok(Json(serde_json::json!({ "otks": app_state
        .get_otks(device_id)
        .await?
        .into_iter()
        .map(|k| BASE64_STANDARD_NO_PAD.encode(&k.otk))
        .collect::<Vec<String>>() })))
}

#[tracing::instrument(skip(app_state, inbound_otks))]
pub async fn upload_otks<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
    Path(device_id): Path<DeviceId>,
    Json(inbound_otks): Json<InboundOtks>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    app_state
        .upload_otks(&user, device_id, inbound_otks)
        .await?;
    Ok(Json(serde_json::json!({ "status": "success "})))
}

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(app_state))]
pub async fn get_user_device_otk<A, D, O, R, W>(
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
    let user = app_state
        .get_user_info(user_id)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    Ok(Json(app_state.get_user_otk(&user, device_id).await?))
}
