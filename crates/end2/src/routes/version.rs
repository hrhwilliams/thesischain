use axum::{Json, response::IntoResponse};

pub async fn version() -> impl IntoResponse {
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "commit": std::env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string()),
    }))
}
