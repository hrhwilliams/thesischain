use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use uuid::Uuid;

use crate::{ApiError, AppState, InboundOtks, User};

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
    app_state.upload_otks(user, device_id, otks).await?;
    Ok(Json(serde_json::json!({ "status": "success "})))
}
