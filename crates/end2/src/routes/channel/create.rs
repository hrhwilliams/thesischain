use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    ApiError, AppError, AppState, InboundChatMessage, MessageId, OutboundChatMessage, User, WsEvent,
};

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
        .auth
        .get_user_by_username(&recipient)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    let response = app_state
        .relay
        .create_channel_between(&user, &recipient)
        .await?;

    app_state
        .relay
        .notify_user(&user, WsEvent::ChannelCreated(response.clone()))
        .await;
    app_state
        .relay
        .notify_user(&recipient, WsEvent::ChannelCreated(response.clone()))
        .await;

    Ok(Json(response))
}

#[tracing::instrument(skip(app_state))]
pub async fn send_message(
    State(app_state): State<AppState>,
    user: User,
    Path(channel_id): Path<Uuid>,
    Json(message): Json<InboundChatMessage>,
) -> Result<impl IntoResponse, ApiError> {
    if channel_id != message.channel_id {
        return Err(AppError::UserError("channel_id mismatch".to_string()).into());
    }

    let sender_device_id = message.device_id;
    let (saved_message, payloads) = app_state.relay.save_message(&user, message).await?;

    // Notify each recipient device with their specific ciphertext
    for payload in payloads {
        if let Some(recipient) = app_state
            .relay
            .get_broadcaster_for_device(payload.recipient_device_id)
            .await
        {
            let outbound = OutboundChatMessage {
                author_id: saved_message.sender_id,
                message_id: saved_message.id,
                device_id: saved_message.sender_device_id,
                channel_id: saved_message.channel_id,
                ciphertext: payload.ciphertext,
                timestamp: saved_message.created,
                is_pre_key: payload.is_pre_key,
            };
            let _ = recipient.send(WsEvent::Message(outbound)).await;
        }
    }

    // Send confirmation to the sender's device
    if let Some(sender) = app_state
        .relay
        .get_broadcaster_for_device(sender_device_id)
        .await
    {
        let _ = sender
            .send(WsEvent::MessageReceived(MessageId {
                message_id: saved_message.id,
                channel_id: saved_message.channel_id,
                timestamp: saved_message.created,
            }))
            .await;
    }

    Ok(())
}
