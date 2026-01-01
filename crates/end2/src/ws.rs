use axum::extract::ws::WebSocket;
use uuid::Uuid;

use crate::AppState;

pub async fn handle_socket(mut socket: WebSocket, session_id: Uuid, app_state: AppState) {
    loop {}
}
