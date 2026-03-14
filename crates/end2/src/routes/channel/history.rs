use crate::{
    ApiError, AppState, AuthService, ChannelId, DeviceKeyService, MessageId, MessageRelayService,
    OtkService, User, WebSessionService,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HistoryRequest {
    pub device: crate::DeviceId,
    pub after: Option<MessageId>,
}

#[tracing::instrument(skip(app_state))]
pub async fn get_channel_history<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
    Path(channel_id): Path<ChannelId>,
    Query(HistoryRequest { device, after }): Query<HistoryRequest>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    Ok(Json(
        app_state
            .get_channel_history(&user, channel_id, device, after)
            .await?,
    ))
}
