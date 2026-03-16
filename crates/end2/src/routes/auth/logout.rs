use crate::{
    ApiError, AppState, AuthService, DeviceKeyService, MessageRelayService, OtkService, User,
    UserId, WebSession, WebSessionService,
};
use axum::{Json, extract::State, response::IntoResponse};

#[tracing::instrument(skip(app_state))]
pub async fn logout<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    web_session: WebSession,
    user: User,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    app_state
        .remove_from_session::<UserId>(web_session, "user_id")
        .await?;

    Ok(Json(serde_json::json!({ "status": "success" })))
}
