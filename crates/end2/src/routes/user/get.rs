use crate::{ApiError, AuthService, UserId};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};

#[tracing::instrument(skip(auth))]
pub async fn get_user_info(
    State(auth): State<impl AuthService>,
    Path(user_id): Path<UserId>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(auth.get_user_info(user_id).await?))
}
