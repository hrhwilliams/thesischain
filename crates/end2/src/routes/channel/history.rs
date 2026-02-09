use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{ApiError, AppState, User};

#[derive(Deserialize)]
pub struct HistoryRequest {
    pub device: Uuid,
    pub after: Option<Uuid>,
}

pub async fn get_channel_history(
    State(app_state): State<AppState>,
    user: User,
    Path(channel_id): Path<Uuid>,
    Query(HistoryRequest { device, after }): Query<HistoryRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(
        app_state.get_channel_history(&user, channel_id, device, after).await?,
    ))
}
