use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{ApiError, AppState};

#[derive(Deserialize)]
pub struct GetUser {
    pub username: String,
}

pub async fn find_user(
    State(app_state): State<AppState>,
    Path(GetUser { username }): Path<GetUser>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(app_state.get_user_by_username(username).await?))
}
