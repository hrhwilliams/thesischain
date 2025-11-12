use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{api::errors::ApiError, app::AppState};

#[derive(Deserialize)]
pub struct Register {
    pub name: String,
    pub key: String,
}

#[derive(Serialize)]
pub struct GetValue {
    pub value: String,
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

pub async fn get_value(
    State(app_state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(key) = query.get("key") {
        let value = app_state
            .swarm
            .get_value(key.clone())
            .await
            .map_err(|e| ApiError::KademliaError(e.to_string()))?;

        if let Some(value) = value {
            return Ok(Json(GetValue { value }));
        }
    }

    Err(ApiError::NotFound)
}
