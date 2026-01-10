use crate::{ApiError, AppState, InboundChatMessage, OutboundChatMessage, User, WsEvent};
use axum::{
    extract::{
        Path, State, WebSocketUpgrade, ws::{Message, WebSocket}
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast::error::RecvError, mpsc};
use tokio::time::{Duration, timeout};
use uuid::Uuid;

#[tracing::instrument(skip(app_state, ws))]
pub async fn handle_websocket(
    State(app_state): State<AppState>,
    user: User,
    Path(device_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApiError> {
    Ok(ws.on_upgrade(move |socket| websocket(socket, user, device_id, app_state)))
}

#[tracing::instrument(skip(socket, app_state))]
pub async fn websocket(socket: WebSocket, user: User, device_id: Uuid, app_state: AppState) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    let user_tx = app_state.get_broadcaster(&user).await;
    let mut user_rx = user_tx.subscribe();

    let (device_tx, mut device_rx) = mpsc::channel::<WsEvent>(32);

    app_state.register_device(device_id, device_tx).await;

    loop {
        tokio::select! {
            read = ws_rx.next() => {
                match read {
                    Some(Ok(Message::Text(text))) => {
                        let msg = match serde_json::from_str::<InboundChatMessage>(&text) {
                            Ok(msg) => msg,
                            Err(e) => {
                                tracing::warn!("invalid inbound chat message in websocket: '{}'", e);
                                continue;
                            }
                        };

                        let (message, devices_payload) = match app_state.save_message(&user, msg).await {
                            Ok(msg) => msg,
                            Err(e) => {
                                tracing::error!("failed to save message {:?}", e);
                                break;
                            }
                        };

                        tracing::info!("got message and {} payloads for it", devices_payload.len());
                        for payload in devices_payload {
                            if let Some(recipient) = app_state.get_broadcaster_for_device(payload.recipient_device_id).await {
                                let outbound = OutboundChatMessage {
                                    message_id: message.id,
                                    device_id: message.sender_device_id,
                                    ciphertext: payload.ciphertext,
                                    timestamp: message.created,
                                    is_pre_key: payload.is_pre_key,
                                };

                                let _ = recipient.send(WsEvent::Message(outbound)).await;
                                tracing::info!("sending message");
                            } else {
                                tracing::warn!("message for unregistered device");
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("closing websocket");
                        break;
                    }
                    _ => {}
                }
            },
            event = user_rx.recv() => {
                match event {
                    Ok(event) => {
                        let json = match serde_json::to_string(&event) {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::error!("failed to serialize into websocket message {}", e);
                                continue;
                            }
                        };

                        if timeout(Duration::from_secs(5), ws_tx.send(Message::Text(json.into()))).await.is_err() {
                            tracing::warn!("timed out sending websocket message");
                            break;
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        tracing::info!("client lagged {} messages", n);
                    }
                    Err(_) => {
                        tracing::info!("closing websocket");
                        break;
                    }
                }
            },
            Some(event) = device_rx.recv() => {
                let json = match serde_json::to_string(&event) {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("failed to serialize into websocket message {}", e);
                        continue;
                    }
                };

                if timeout(Duration::from_secs(5), ws_tx.send(Message::Text(json.into()))).await.is_err() {
                    tracing::warn!("timed out sending websocket message");
                    break;
                }

                tracing::info!("sent message");
            }
        }
    }

    app_state.unregister_device(device_id).await;
}
