use crate::{
    ApiError, AppState, AuthService, ChannelId, DeviceKeyService, MessageRelayService, OtkService,
    User, WebSessionService,
};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};

#[tracing::instrument(skip(app_state))]
pub async fn get_channel_info<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
    Path(channel_id): Path<ChannelId>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    Ok(Json(app_state.get_channel_info(&user, channel_id).await?))
}

#[tracing::instrument(skip(app_state))]
pub async fn get_all_channels<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    Ok(Json(app_state.get_user_channels(&user).await?))
}
