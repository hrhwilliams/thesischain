use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{ApiError, AppError, AppState, User};

#[derive(Deserialize)]
pub struct KeyRequest {
    pub receiver: String,
}

#[tracing::instrument(skip(app_state))]
pub async fn get_identity_key(
    State(app_state): State<AppState>,
    _user: User,
    Path(KeyRequest { receiver }): Path<KeyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let receiver = app_state
        .get_user_by_username(receiver)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    Ok(Json(app_state.get_identity_key(receiver).await))
}

#[tracing::instrument(skip(app_state))]
pub async fn get_otk(
    State(app_state): State<AppState>,
    _user: User,
    Path(KeyRequest { receiver }): Path<KeyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let receiver = app_state
        .get_user_by_username(receiver)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    Ok(Json(app_state.get_otk(receiver).await?))
}

pub async fn count_otks(
    State(app_state): State<AppState>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    let count = app_state.count_otks(user).await?;
    Ok(Json(serde_json::json!({ "count": count })))
}

#[derive(Deserialize)]
pub struct OtkUpload {
    pub keys: Vec<String>,
}

pub async fn publish_otks(
    State(app_state): State<AppState>,
    user: User,
    Json(OtkUpload { keys }): Json<OtkUpload>,
) -> Result<impl IntoResponse, ApiError> {
    app_state.publish_otks(user, keys).await?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}
