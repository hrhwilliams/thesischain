use axum::{Json, Router, extract::State, http::StatusCode, response::Html, routing::get};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Serialize)]
struct Peer {
    address: String,
}

#[derive(Clone)]
struct AppState {
    peers: Arc<RwLock<Vec<Peer>>>,
    port: u16,
}

impl AppState {
    fn port(&self) -> u16 {
        self.port
    }
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("localhost:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    println!("Server running on http://localhost:{}", port);

    let shared_state = AppState {
        peers: Arc::new(RwLock::new(vec![])),
        port,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/add_peer", get(add_peer).post(add_peer_post))
        .route("/get_peers", get(get_peers))
        .route("/health", get(health))
        // .route("/chain", get(chain))
        .with_state(shared_state);

    axum::serve(listener, app).await.unwrap();
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let peers = {
        let peers = state.peers.read().await;
        peers.clone()
    };

    let peers_list = peers
        .iter()
        .map(|peer| {
            format!(
                r#"<li><a href="{}">{}</a></li>"#,
                peer.address, peer.address
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en-US">
<head>
    <meta content-type="text/html" charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Node {}</title>
</head>
<body>
    <h1>Node {} - Index</h1>
    <p><a href="/chain">Chain</a> | <a href="/add_peer">Add Peer</a></p>
    <br>
    <ul>
    {}
    </ul>
</body>
</html>"#,
        state.port(),
        state.port(),
        peers_list
    ))
}

async fn add_peer(State(state): State<AppState>) -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en-US">
<head>
    <meta content-type="text/html" charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Node {} - Add Peer</title>
</head>
<body>
    <h1>Node {} - Add Peer</h1>
    <form action="/add_peer" method="post">
        <label for="port">Peer Port:</label>
        <input type="text" id="port" name="port" required>
        <button type="submit">Add Peer</button>
    </form>
    <p><a href="/">Home</a></p>
</body>
</html>"#,
        state.port(),
        state.port()
    ))
}

async fn add_peer_post(
    State(state): State<AppState>,
    axum::extract::Form(form): axum::extract::Form<HashMap<String, String>>,
) -> Result<Html<String>, StatusCode> {
    if let Some(port) = form.get("port") {
        if let Ok(port) = port.parse::<u16>() {
            {
                let mut peers = state.peers.write().await;
                peers.push(Peer {
                    address: format!("http://localhost:{}", port),
                });
            }

            Ok(index(State(state)).await)
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// Return a JSON of peers this host knows
#[axum::debug_handler]
async fn get_peers(State(state): State<AppState>) -> Json<Vec<Peer>> {
    let peers = {
        let peers = state.peers.read().await;
        peers.clone()
    };

    Json(peers)
}

async fn health() -> StatusCode {
    StatusCode::OK
}
