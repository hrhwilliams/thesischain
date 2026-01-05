use axum::{
    Json,
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use serde::Deserialize;
use uuid::Uuid;

use crate::{ApiError, AppError, AppState, User, channel_socket};

pub async fn get_all_channels(
    State(app_state): State<AppState>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(app_state.get_user_channels(user).await?))
}

#[derive(Deserialize)]
pub struct ChannelWith {
    pub receiver: String,
}

#[tracing::instrument(skip(app_state))]
pub async fn create_channel_with(
    State(app_state): State<AppState>,
    user: User,
    Path(ChannelWith { receiver }): Path<ChannelWith>,
) -> Result<impl IntoResponse, ApiError> {
    let receiver = app_state
        .get_user_by_username(receiver)
        .await?
        .ok_or(AppError::NoSuchUser)?;
    Ok(Json(
        app_state.create_channel_between(user, receiver).await?,
    ))
}

#[derive(Deserialize)]
pub struct WebsocketParams {
    pub channel_id: Uuid,
}

#[tracing::instrument(skip(app_state, ws))]
pub async fn handle_websocket(
    State(app_state): State<AppState>,
    Path(WebsocketParams { channel_id }): Path<WebsocketParams>,
    user: User,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApiError> {
    let other = app_state
        .get_other_channel_participant(&user, channel_id)
        .await?;
    Ok(ws.on_upgrade(move |socket| channel_socket(socket, user, other, channel_id, app_state)))
}

#[tracing::instrument(skip(app_state))]
pub async fn get_channel_participant_info(
    State(app_state): State<AppState>,
    Path(WebsocketParams { channel_id }): Path<WebsocketParams>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    let other = app_state
        .get_other_channel_participant(&user, channel_id)
        .await?;
    Ok(Json(serde_json::json!({
        "username": other.username,
        "curve25519": BASE64_STANDARD_NO_PAD.encode(other.curve25519)
    })))
}
