use axum::{Json, extract::State, response::IntoResponse};
use serde::Deserialize;

use crate::{ApiError, AuthService, MessageRelayService, NewNickname, User, WsEvent};

/// returns user struct if the user has the Session cookie set with a valid session token
#[tracing::instrument]
pub async fn me(user: User) -> impl IntoResponse {
    Json(user)
}

#[derive(Deserialize)]
pub struct Nickname {
    pub nickname: String,
}

#[tracing::instrument(skip(auth, relay))]
pub async fn change_nickname(
    State(auth): State<impl AuthService>,
    State(relay): State<impl MessageRelayService>,
    user: User,
    Json(Nickname { nickname }): Json<Nickname>,
) -> Result<impl IntoResponse, ApiError> {
    auth.change_nickname(&user, &nickname).await?;

    let users_to_notify = auth.get_known_users(&user).await?;
    for other in users_to_notify {
        relay
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
