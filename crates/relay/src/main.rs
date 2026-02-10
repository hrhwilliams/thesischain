use axum::{
    Json,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use defs::Identity;
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::net::TcpListener;

const MINER_URL: &'static str = "url";

#[derive(Clone, Default)]
struct AppState;

async fn register(
    State(_app_state): State<AppState>,
    Json(payload): Json<Identity>,
) -> Result<impl IntoResponse, StatusCode> {
    let client = reqwest::Client::new();
    let response = client
        .post(MINER_URL)
        .json(&payload)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok("OK".into_response())
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();

    let port = std::env::var("RELAY_PORT")
        .expect("Missing RELAY_PORT")
        .parse()
        .expect("RELAY_PORT must be an integer in the range 0-65535");

    let listener = TcpListener::bind(("localhost", port))
        .await
        .expect("listen");

    let router = axum::Router::new()
        .route("/api/register", post(register))
        .with_state(AppState::default());

    axum::serve(listener, router.into_make_service()).await
}
