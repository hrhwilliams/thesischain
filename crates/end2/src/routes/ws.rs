use crate::{
    ApiError, AppState, AuthService, CountedEvent, DeviceId, DeviceKeyService, MessageRelayService,
    OtkService, ReplayRequest, User, WebSessionService, WsEvent,
};
use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::{broadcast::error::RecvError, mpsc};
use tokio::time::{Duration, interval, timeout};

#[tracing::instrument(skip(app_state, ws))]
pub async fn handle_websocket<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
    Path(device_id): Path<DeviceId>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone + 'static,
    W: WebSessionService + Clone,
{
    let relay = app_state.relay;
    Ok(ws.on_upgrade(move |socket| websocket(socket, user, device_id, relay)))
}

/// Serializes a `CountedEvent`, pushes it to history, and sends it over the websocket.
/// Returns `Ok(())` on success, `Err(true)` if sending timed out (caller should break),
/// or continues silently on serialization failure.
async fn send_event(
    ws_tx: &mut futures::stream::SplitSink<WebSocket, Message>,
    history: &mut Vec<CountedEvent>,
    counted: CountedEvent,
) -> Result<(), ()> {
    let json = match serde_json::to_string(&counted) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to serialize websocket message: {e}");
            return Ok(());
        }
    };

    history.push(counted);

    if timeout(
        Duration::from_secs(5),
        ws_tx.send(Message::Text(json.into())),
    )
    .await
    .is_err()
    {
        tracing::warn!("timed out sending websocket message");
        return Err(());
    }

    Ok(())
}

#[tracing::instrument(skip(socket, relay))]
pub async fn websocket(
    socket: WebSocket,
    user: User,
    device_id: DeviceId,
    relay: impl MessageRelayService,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    let user_tx = relay.get_broadcaster(&user).await;
    let mut user_rx = user_tx.subscribe();

    let (device_tx, mut device_rx) = mpsc::channel::<WsEvent>(32);

    relay.register_device(device_id, device_tx).await;

    let mut next_counter: u64 = 0;
    let mut history: Vec<CountedEvent> = Vec::new();
    let mut ping_interval = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                if ws_tx.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
            },
            read = ws_rx.next() => {
                match read {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(req) = serde_json::from_str::<ReplayRequest>(&text) {
                            #[allow(clippy::cast_sign_loss)]
                            let from = if req.replay < 0 { 0 } else { req.replay as u64 + 1 };
                            tracing::info!("replaying events from counter {from}");

                            for event in &history {
                                if event.counter >= from {
                                    let json = match serde_json::to_string(event) {
                                        Ok(s) => s,
                                        Err(e) => {
                                            tracing::error!("failed to serialize replay event: {e}");
                                            continue;
                                        }
                                    };

                                    if timeout(Duration::from_secs(5), ws_tx.send(Message::Text(json.into()))).await.is_err() {
                                        tracing::warn!("timed out sending replay message");
                                        break;
                                    }
                                }
                            }
                        } else {
                            tracing::warn!("unexpected text message on websocket: '{text}'");
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
                        let counted = CountedEvent { counter: next_counter, event };
                        next_counter += 1;

                        if send_event(&mut ws_tx, &mut history, counted).await.is_err() {
                            break;
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        tracing::info!("client lagged {n} messages");
                    }
                    Err(_) => {
                        tracing::info!("closing websocket");
                        break;
                    }
                }
            },
            Some(event) = device_rx.recv() => {
                let counted = CountedEvent { counter: next_counter, event };
                next_counter += 1;

                if send_event(&mut ws_tx, &mut history, counted).await.is_err() {
                    break;
                }
            }
        }
    }

    relay.unregister_device(device_id).await;
}
