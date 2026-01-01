// use axum::extract::ws::{Message, WebSocket};
// use futures::{SinkExt, StreamExt};

// use crate::{AppState, ChatMessage, RoomId, Session};

// pub async fn handle_socket(
//     socket: WebSocket,
//     session: Session,
//     room_id: RoomId,
//     app_state: AppState,
// ) {
//     let (mut sender, mut receiver) = socket.split();

//     let room = app_state.get_room(room_id).await;
//     let history = room.history().await;
//     let mut rx = room.subscribe();

//     let me_send = session.username().clone();
//     let me_recv = session.username().clone();

//     let mut send_task = tokio::spawn(async move {
//         for message in history {
//             let json = serde_json::to_string(&message).expect("message not valid JSON");
//             sender.send(Message::Text(json.into())).await;
//         }
//         while let Ok(msg) = rx.recv().await {
//             // if msg.user != me_send {
//             let json = serde_json::to_string(&msg).expect("message not valid JSON");
//             sender.send(Message::Text(json.into())).await;
//             // }
//         }
//     });

//     let mut recv_task = tokio::spawn(async move {
//         while let Some(Ok(Message::Text(text))) = receiver.next().await {
//             if let Ok(mut chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
//                 chat_msg.user = me_recv.clone();

//                 {
//                     let mut room_guard = room.history.write().await;
//                     room_guard.push(chat_msg.clone());
//                 }

//                 let _ = room.send(chat_msg);
//             }
//         }
//     });

//     tokio::select! {
//         _ = (&mut send_task) => recv_task.abort(),
//         _ = (&mut recv_task) => send_task.abort(),
//     };
// }
