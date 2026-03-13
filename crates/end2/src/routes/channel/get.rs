use crate::{ApiError, ChannelId, MessageRelayService, User};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};

#[tracing::instrument(skip(relay))]
pub async fn get_channel_info(
    State(relay): State<impl MessageRelayService>,
    user: User,
    Path(channel_id): Path<ChannelId>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(relay.get_channel_info(&user, channel_id).await?))
}

#[tracing::instrument(skip(relay))]
pub async fn get_all_channels(
    State(relay): State<impl MessageRelayService>,
    user: User,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(relay.get_user_channels(&user).await?))
}
