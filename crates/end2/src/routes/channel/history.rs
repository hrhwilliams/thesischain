use crate::{ApiError, ChannelId, DeviceId, MessageId, MessageRelayService, User};
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

#[tracing::instrument(skip(relay))]
pub async fn get_channel_history(
    State(relay): State<impl MessageRelayService>,
    user: User,
    Path(channel_id): Path<ChannelId>,
    Query(HistoryRequest { device, after }): Query<HistoryRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(
        relay
            .get_channel_history(&user, channel_id, device, after)
            .await?,
    ))
}
