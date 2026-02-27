use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(info): Json<miner::MinerInfo>,
) -> impl IntoResponse {
    let mut miners = state.miners.write().await;

    // Replace existing entry for the same peer_id, or add new
    if let Some(existing) = miners.iter_mut().find(|m| m.peer_id == info.peer_id) {
        *existing = info;
    } else {
        miners.push(info);
    }

    StatusCode::OK
}

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
    let miners = state.miners.read().await;
    Json(miners.clone())
}
