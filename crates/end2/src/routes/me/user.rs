use axum::{Json, extract::State, response::IntoResponse};
use serde::Deserialize;

use crate::{
    ApiError, AppState, AuthService, DeviceKeyService, MessageRelayService, NewNickname, OtkService,
    User, WebSessionService, WsEvent,
};

/// returns user struct if the user has the Session cookie set with a valid session token
#[tracing::instrument]
pub async fn me(user: User) -> impl IntoResponse {
    Json(user)
}

#[derive(Deserialize)]
pub struct Nickname {
    pub nickname: String,
}

#[tracing::instrument(skip(app_state))]
pub async fn change_nickname<A, D, O, R, W>(
    State(app_state): State<AppState<A, D, O, R, W>>,
    user: User,
    Json(Nickname { nickname }): Json<Nickname>,
) -> Result<impl IntoResponse, ApiError>
where
    A: AuthService + Clone,
    D: DeviceKeyService + Clone,
    O: OtkService + Clone,
    R: MessageRelayService + Clone,
    W: WebSessionService + Clone,
{
    app_state.change_nickname(&user, &nickname).await?;

    let users_to_notify = app_state.get_known_users(&user).await?;
    for other in users_to_notify {
        app_state
            .notify_user(
                &other,
                WsEvent::NicknameChanged(NewNickname {
                    user_id: user.id,
                    nickname: nickname.clone(),
                }),
            )
            .await;
    }

    Ok(Json(serde_json::json!({ "status": "success" })))
}
