use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{ApiError, AppState, NewUser, User};

/// GET `/api/auth/me`
///
/// returns user struct if the user has the Session cookie set with a valid session token
// #[tracing::instrument]
// pub async fn me(user: User) -> impl IntoResponse {
//     Json(user)
// }

/// POST `/api/auth/register` with JSON payload `{ username: string, ed25519: bytes, curve25519: bytes, signature: bytes}`
#[tracing::instrument(skip(app_state))]
pub async fn register(
    State(app_state): State<AppState>,
    Json(new_user): Json<NewUser>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(app_state.register_user(new_user).await?))
}

#[derive(Deserialize)]
pub struct ChallengeRequest {
    pub user: String,
}

/// GET `/api/auth/challenge?user={username}`
#[tracing::instrument(skip(app_state))]
pub async fn get_challenge(
    State(app_state): State<AppState>,
    Query(ChallengeRequest { user }): Query<ChallengeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(app_state.generate_challenge_for(user).await?))
}

#[derive(Deserialize)]
pub struct ChallengeResponse {
    pub id: Uuid,
    pub signature: String,
}

/// POST `/api/auth/challenge with JSON payload { id: uuid, response: bytes }`
#[tracing::instrument(skip(app_state))]
pub async fn post_challenge(
    State(app_state): State<AppState>,
    Json(ChallengeResponse { id, signature }): Json<ChallengeResponse>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(
        app_state
            .verify_response_and_create_session(id, signature)
            .await?,
    ))
}
