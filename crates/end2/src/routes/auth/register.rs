use axum::{Json, extract::State, response::IntoResponse};

use crate::{
    ApiError, AppState, AuthService, DeviceKeyService, InboundUser, MessageRelayService,
    OtkService, WebSession, WebSessionService,
};

#[tracing::instrument(skip(app_state, new_user), fields(user = %new_user.username))]
pub async fn register<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    web_session: Option<WebSession>,
    Json(new_user): Json<InboundUser>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    let user = app_state.register_user(new_user).await?;

    if let Some(web_session) = web_session {
        app_state
            .insert_into_session(web_session, "user_id".to_string(), user.id)
            .await?;
    }

    Ok(Json(user))
}
