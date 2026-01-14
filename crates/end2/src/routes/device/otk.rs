use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use uuid::Uuid;

use crate::{ApiError, AppError, AppState, InboundOtks, User};

#[tracing::instrument(skip(app_state))]
pub async fn get_otks(
    State(app_state): State<AppState>,
    _user: User,
    Path(device_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let otks = app_state
        .get_otks(device_id)
        .await?
        .into_iter()
        .map(|k| BASE64_STANDARD_NO_PAD.encode(&k.otk))
        .collect::<Vec<String>>();
    Ok(Json(serde_json::json!({ "otks": otks })))
}

#[tracing::instrument(skip(app_state, otks))]
pub async fn upload_otks(
    State(app_state): State<AppState>,
    user: User,
    Path(device_id): Path<Uuid>,
    Json(otks): Json<InboundOtks>,
) -> Result<impl IntoResponse, ApiError> {
    app_state.upload_otks(&user, device_id, otks).await?;
    Ok(Json(serde_json::json!({ "status": "success "})))
}

#[tracing::instrument(skip(app_state))]
pub async fn get_user_device_otk(
    State(app_state): State<AppState>,
    _user: User,
    Path((user_id, device_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ApiError> {
    let user = app_state
        .get_user_info(user_id)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    Ok(Json(app_state.get_user_otk(&user, device_id).await?))
}
