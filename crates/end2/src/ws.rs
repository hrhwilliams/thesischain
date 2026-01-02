use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppState, NewChatMessage, User};

#[derive(Deserialize)]
struct RecvChatMessage {
    pub content: String,
}

#[tracing::instrument(skip(socket, app_state))]
pub async fn handle_socket(socket: WebSocket, user: User, room_id: Uuid, app_state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    let tx = app_state.get_channel(room_id).await;
    let mut rx = tx.subscribe();

    let app_state_recv = app_state.clone();

    let mut send_task = tokio::spawn(async move {
        if let Ok(history) = app_state.get_room_history(room_id).await {
            for msg in history {
                if let Ok(json) = serde_json::to_string(&msg) {
                    let msg = Message::Text(json.into());
                    let dbg_msg = msg.clone();
                    if sender.send(msg).await.is_err() {
                        tracing::error!("bad message {:?}", dbg_msg);
                        return;
                    }
                }
            }
        }

        while let Ok(chat_msg) = rx.recv().await {
            tracing::debug!("got message {:?}", chat_msg);
            if let Ok(json) = serde_json::to_string(&chat_msg) {
                let msg = Message::Text(json.into());
                let dbg_msg = msg.clone();
                if sender.send(msg).await.is_err() {
                    tracing::error!("bad message {:?}", dbg_msg);
                    break;
                }
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            tracing::debug!("got message {}", text.to_string());
            if let Ok(incoming) = serde_json::from_str::<RecvChatMessage>(&text) {
                let new_chat_message = NewChatMessage {
                    room_id,
                    author: user.id,
                    content: incoming.content
                };

                match app_state_recv.save_message(new_chat_message).await {
                    Ok(saved_msg) => {
                        let _ = tx.send(saved_msg);
                    }
                    Err(e) => {
                        tracing::error!("Failed to save message: {:?}", e);
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
