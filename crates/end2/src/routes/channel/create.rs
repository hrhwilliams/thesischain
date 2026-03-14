use crate::{
    ApiError, AppError, AppState, AuthService, ChannelId, DeviceKeyService, InboundChatMessage,
    MessageReceipt, MessageRelayService, OtkService, OutboundChatMessage, User, WebSessionService,
    WsEvent,
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

#[tracing::instrument(skip(app_state))]
pub async fn create_channel_with<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
    Json(ChannelWith { recipient }): Json<ChannelWith>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let recipient = app_state
        .get_user_by_username(&recipient)
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

#[tracing::instrument(skip(app_state))]
pub async fn send_message<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
    Path(channel_id): Path<ChannelId>,
    Json(message): Json<InboundChatMessage>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    if channel_id != message.channel_id {
        return Err(AppError::UserError("channel_id mismatch".to_string()).into());
    }

    let sender_device_id = message.device_id;
    let (saved_message, payloads) = app_state.save_message(&user, message).await?;

    // Notify each recipient device with their specific ciphertext
    for payload in payloads {
        if let Some(recipient) = app_state
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
        .get_broadcaster_for_device(sender_device_id)
        .await
    {
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
