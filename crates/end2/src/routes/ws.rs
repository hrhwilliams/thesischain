use crate::{ApiError, AppState, InboundChatMessage, User, WsEvent};
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast::error::RecvError;
use tokio::time::{Duration, timeout};

#[tracing::instrument(skip(app_state, ws))]
pub async fn handle_websocket(
    State(app_state): State<AppState>,
    user: User,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApiError> {
    Ok(ws.on_upgrade(move |socket| websocket(socket, user, app_state)))
}

#[tracing::instrument(skip(socket, app_state))]
pub async fn websocket(socket: WebSocket, user: User, app_state: AppState) {
    let (mut to_client, mut from_client) = socket.split();

    let tx = app_state.get_broadcaster(&user).await;
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            incoming = from_client.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        let msg = match serde_json::from_str::<InboundChatMessage>(&text) {
                            Ok(msg) => msg,
                            Err(e) => {
                                tracing::warn!("invalid inbound chat message in websocket: '{}'", e);
                                continue;
                            }
                        };

                        let message = match app_state.save_message(&user, msg).await {
                            Ok(msg) => msg,
                            Err(e) => {
                                tracing::error!("failed to save message {:?}", e);
                                break;
                            }
                        };

                        let recipient = app_state.get_broadcaster(message.recipient_id).await;
                        let _ = recipient.send(WsEvent::Message(message.into()));
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        tracing::info!("closing websocket");
                        break;
                    }
                    _ => {}
                }
            },
            outgoing = rx.recv() => {
                match outgoing {
                    Ok(event) => {
                        let json = match serde_json::to_string(&event) {
                            Ok(s) => s,
                            Err(e) => {
                                tracing::error!("failed to serialize into websocket message {}", e);
                                continue;
                            }
                        };

                        if timeout(Duration::from_secs(5), to_client.send(Message::Text(json.into()))).await.is_err() {
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
            }
        }
    }
}
