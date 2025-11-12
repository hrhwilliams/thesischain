use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::{api::errors::ApiError, app::AppState};

#[derive(Deserialize)]
pub struct Register {
    pub name: String,
    pub key: String,
}

pub async fn register(
    State(app_state): State<AppState>,
    Json(json): Json<Register>,
) -> Result<impl IntoResponse, ApiError> {
    app_state
        .swarm
        .put(json.name, json.key)
        .await
        .map_err(|e| ApiError::KademliaError(e.to_string()))?;

    Ok(StatusCode::OK)
}
