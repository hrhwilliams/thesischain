use crate::{ApiError, AppError, AuthService, DeviceId, InboundOtks, OtkService, User, UserId};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(otks))]
pub async fn get_otks(
    State(otks): State<impl OtkService>,
    _user: User,
    Path(device_id): Path<DeviceId>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(serde_json::json!({ "otks": otks
        .get_otks(device_id)
        .await?
        .into_iter()
        .map(|k| BASE64_STANDARD_NO_PAD.encode(&k.otk))
        .collect::<Vec<String>>() })))
}

#[tracing::instrument(skip(otks, inbound_otks))]
pub async fn upload_otks(
    State(otks): State<impl OtkService>,
    user: User,
    Path(device_id): Path<DeviceId>,
    Json(inbound_otks): Json<InboundOtks>,
) -> Result<impl IntoResponse, ApiError> {
    otks.upload_otks(&user, device_id, inbound_otks).await?;
    Ok(Json(serde_json::json!({ "status": "success "})))
}

#[allow(clippy::used_underscore_binding)]
#[tracing::instrument(skip(auth, otks))]
pub async fn get_user_device_otk(
    State(auth): State<impl AuthService>,
    State(otks): State<impl OtkService>,
    _user: User,
    Path((user_id, device_id)): Path<(UserId, DeviceId)>,
) -> Result<impl IntoResponse, ApiError> {
    let user = auth
        .get_user_info(user_id)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    Ok(Json(otks.get_user_otk(&user, device_id).await?))
}
