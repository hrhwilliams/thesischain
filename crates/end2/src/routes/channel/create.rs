use crate::{
    ApiError, AppError, AuthService, ChannelId, InboundChatMessage, MessageReceipt,
    MessageRelayService, OutboundChatMessage, User, WsEvent,
};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChannelWith {
    pub recipient: String,
}

#[tracing::instrument(skip(auth, relay))]
pub async fn create_channel_with(
    State(auth): State<impl AuthService>,
    State(relay): State<impl MessageRelayService>,
    user: User,
    Json(ChannelWith { recipient }): Json<ChannelWith>,
) -> Result<impl IntoResponse, ApiError> {
    let recipient = auth
        .get_user_by_username(&recipient)
        .await?
        .ok_or(AppError::NoSuchUser)?;

    let response = relay.create_channel_between(&user, &recipient).await?;

    relay
        .notify_user(&user, WsEvent::ChannelCreated(response.clone()))
        .await;
    relay
        .notify_user(&recipient, WsEvent::ChannelCreated(response.clone()))
        .await;

    Ok(Json(response))
}

#[tracing::instrument(skip(relay))]
pub async fn send_message(
    State(relay): State<impl MessageRelayService>,
    user: User,
    Path(channel_id): Path<ChannelId>,
    Json(message): Json<InboundChatMessage>,
) -> Result<impl IntoResponse, ApiError> {
    if channel_id != message.channel_id {
        return Err(AppError::UserError("channel_id mismatch".to_string()).into());
    }

    let sender_device_id = message.device_id;
    let (saved_message, payloads) = relay.save_message(&user, message).await?;

    // Notify each recipient device with their specific ciphertext
    for payload in payloads {
        if let Some(recipient) = relay
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
    if let Some(sender) = relay.get_broadcaster_for_device(sender_device_id).await {
        let _ = sender
            .send(WsEvent::MessageReceived(MessageReceipt {
                message_id: saved_message.id,
                channel_id: saved_message.channel_id,
                timestamp: saved_message.created,
            }))
            .await;
    }

    Ok(())
}
