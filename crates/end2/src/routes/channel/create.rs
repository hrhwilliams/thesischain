use axum::{Json, extract::State, response::IntoResponse};
use serde::Deserialize;

use crate::{ApiError, AppError, AppState, User, WsEvent};

#[derive(Deserialize)]
pub struct ChannelWith {
    pub recipient: String,
}

#[tracing::instrument(skip(app_state))]
pub async fn create_channel_with(
    State(app_state): State<AppState>,
    user: User,
    Json(ChannelWith { recipient }): Json<ChannelWith>,
) -> Result<impl IntoResponse, ApiError> {
    let recipient = app_state
        .get_user_by_username(recipient)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    let response = app_state.create_channel_between(&user, &recipient).await?;

    app_state
        .notify_user(&user, WsEvent::ChannelCreated(response.clone()))
        .await;
    app_state
        .notify_user(&recipient, WsEvent::ChannelCreated(response.clone()))
        .await;

    Ok(Json(response))
}

// #[tracing::instrument(skip(app_state))]
// pub async fn send_message(
//     State(app_state): State<AppState>,
//     user: User,
//     Path(channel_id): Path<Uuid>,
//     Json(message): Json<InboundChatMessage>,
// ) -> Result<impl IntoResponse, ApiError> {
//     todo!()
// }
