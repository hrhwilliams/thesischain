use axum::{Json, response::IntoResponse};

use crate::{OutboundUser, User};

/// returns user struct if the user has the Session cookie set with a valid session token
#[tracing::instrument]
pub async fn me(user: User) -> impl IntoResponse {
    Json(OutboundUser::from(user))
}
