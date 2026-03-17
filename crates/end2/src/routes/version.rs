use axum::{Json, response::IntoResponse};

pub async fn version() -> impl IntoResponse {
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "commit": env!("GIT_COMMIT"),
    }))
}
