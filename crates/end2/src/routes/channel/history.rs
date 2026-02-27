use crate::{ApiError, AppState, ChannelId, DeviceId, MessageId, User};
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HistoryRequest {
    pub device: DeviceId,
    pub after: Option<MessageId>,
}

#[tracing::instrument(skip(app_state))]
pub async fn get_channel_history(
    State(app_state): State<AppState>,
    user: User,
    Path(channel_id): Path<ChannelId>,
    Query(HistoryRequest { device, after }): Query<HistoryRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(
        app_state
            .relay
            .get_channel_history(&user, channel_id, device, after)
            .await?,
    ))
}
