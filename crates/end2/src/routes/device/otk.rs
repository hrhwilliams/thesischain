use crate::{ApiError, AppError, AppState, DeviceId, InboundOtks, User, UserId};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(app_state))]
pub async fn get_otks(
    State(app_state): State<AppState>,
    _user: User,
    Path(device_id): Path<DeviceId>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(serde_json::json!({ "otks": app_state
        .otks
        .get_otks(device_id)
        .await?
        .into_iter()
        .map(|k| BASE64_STANDARD_NO_PAD.encode(&k.otk))
        .collect::<Vec<String>>() })))
}

#[tracing::instrument(skip(app_state, inbound_otks))]
pub async fn upload_otks(
    State(app_state): State<AppState>,
    user: User,
    Path(device_id): Path<DeviceId>,
    Json(inbound_otks): Json<InboundOtks>,
) -> Result<impl IntoResponse, ApiError> {
    app_state.otks.upload_otks(&user, device_id, inbound_otks).await?;
    Ok(Json(serde_json::json!({ "status": "success "})))
}

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(app_state))]
pub async fn get_user_device_otk(
    State(app_state): State<AppState>,
    _user: User,
    Path((user_id, device_id)): Path<(UserId, DeviceId)>,
) -> Result<impl IntoResponse, ApiError> {
    let user = app_state
        .auth
        .get_user_info(user_id)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    Ok(Json(app_state.otks.get_user_otk(&user, device_id).await?))
}
