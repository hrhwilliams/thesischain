use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{ApiError, AppState, User};

#[derive(Deserialize)]
pub struct HistoryRequest {
    pub device_id: Uuid,
}

pub async fn get_channel_history(
    State(app_state): State<AppState>,
    user: User,
    Path(channel_id): Path<Uuid>,
    Json(HistoryRequest { device_id }): Json<HistoryRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(
        app_state
            .get_channel_history(&user, channel_id, device_id)
            .await?,
    ))
}
