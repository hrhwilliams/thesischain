use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, Channel, InboundChatMessage, NewChatMessage, OutboundChatMessage, User};

#[derive(Clone, Debug, Serialize)]
pub enum WsEvent {
    ChannelCreated(Channel),
    Message(String),
}

// #[tracing::instrument(skip(socket, app_state))]
// pub async fn channel_socket(
//     socket: WebSocket,
//     user: User,
//     other_user: User,
//     channel_id: Uuid,
//     app_state: AppState,
// ) {
//     let (mut to_client, mut from_client) = socket.split();

//     let tx = app_state.get_channel_broadcaster(channel_id).await;
//     let mut rx = tx.subscribe();

//     // when user connects, send all unsent messages in the channel

//     let mut from_client_task = tokio::spawn(async move {
//         while let Some(Ok(Message::Text(text))) = from_client.next().await {
//             let message = serde_json::from_str::<InboundChatMessage>(text.as_str()).unwrap();

//             let mut new_chat_message = NewChatMessage::try_from(message).ok().unwrap();
//             new_chat_message.author_id = user.id;
//             tracing::info!("received message {new_chat_message:?}");
//             match app_state.save_message(new_chat_message).await {
//                 Ok(saved_message) => {
//                     if let Err(e) = tx.send(saved_message) {
//                         tracing::error!("{}", e);
//                         return;
//                     }
//                 }
//                 Err(e) => {
//                     tracing::error!("{}", e);
//                     return;
//                 }
//             };
//         }
//     });

//     let mut to_client_task = tokio::spawn(async move {
//         while let Ok(chat_msg) = rx.recv().await {
//             let mut outbound = OutboundChatMessage::from(chat_msg);
//             outbound.author = other_user.username.clone();

//             match serde_json::to_string(&outbound) {
//                 Ok(json) => {
//                     if let Err(e) = to_client.send(Message::Text(json.into())).await {
//                         tracing::error!("{}", e);
//                         return;
//                     }
//                 }
//                 Err(e) => {
//                     tracing::error!("{}", e);
//                     return;
//                 }
//             }
//         }
//     });

// other loop receives via rx

// let app_state_recv = app_state.clone();

// let mut send_task = tokio::spawn(async move {
//     if let Ok(history) = app_state.get_room_history(room_id).await {
//         for msg in history {
//             if let Ok(json) = serde_json::to_string(&msg) {
//                 let msg = Message::Text(json.into());
//                 let dbg_msg = msg.clone();
//                 if sender.send(msg).await.is_err() {
//                     tracing::error!("bad message {:?}", dbg_msg);
//                     return;
//                 }
//             }
//         }
//     }

//     while let Ok(chat_msg) = rx.recv().await {
//         tracing::debug!("got message {:?}", chat_msg);
//         if let Ok(json) = serde_json::to_string(&chat_msg) {
//             let msg = Message::Text(json.into());
//             let dbg_msg = msg.clone();
//             if sender.send(msg).await.is_err() {
//                 tracing::error!("bad message {:?}", dbg_msg);
//                 break;
//             }
//         }
//     }
// });

// let mut recv_task = tokio::spawn(async move {
//     while let Some(Ok(Message::Text(text))) = receiver.next().await {
//         tracing::debug!("got message {}", text.to_string());
//         if let Ok(incoming) = serde_json::from_str::<RecvChatMessage>(&text) {
//             let new_chat_message = NewChatMessage {
//                 room_id,
//                 author: user.id,
//                 content: incoming.content,
//             };

//             match app_state_recv.save_message(new_chat_message).await {
//                 Ok(saved_msg) => {
//                     let _ = tx.send(saved_msg);
//                 }
//                 Err(e) => {
//                     tracing::error!("Failed to save message: {:?}", e);
//                 }
//             }
//         }
//     }
// });

//     tokio::select! {
//         _ = (&mut to_client_task) => from_client_task.abort(),
//         _ = (&mut from_client_task) => to_client_task.abort(),
//     };
// }
