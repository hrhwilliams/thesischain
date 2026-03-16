use crate::{
    ApiError, AppState, AuthService, DeviceKeyService, MessageRelayService, OtkService, UserId,
    WebSessionService,
};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};

#[tracing::instrument(skip(app_state))]
pub async fn get_user_info<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    Path(user_id): Path<UserId>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    Ok(Json(app_state.get_user_info(user_id).await?))
}
