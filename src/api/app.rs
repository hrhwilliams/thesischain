use axum::{http::StatusCode, response::IntoResponse, routing::get};
use tokio::net::TcpListener;

use crate::{api::ApiRoutes, node::EdgeNodes};

#[derive(Clone)]
pub struct AppState {
    pub swarm: EdgeNodes,
}

impl AppState {
    fn new() -> Self {
        Self {
            swarm: EdgeNodes::new().unwrap(),
        }
    }
}

pub struct App {
    listener: TcpListener,
    router: axum::Router,
}

impl App {
    pub fn new(listener: TcpListener) -> Self {
        let app_state = AppState::new();

        let router = axum::Router::new()
            .route("/", get(health_check))
            .nest("/api", ApiRoutes::router())
            .with_state(app_state);

        Self { listener, router }
    }

    pub async fn serve(self) -> Result<(), std::io::Error> {
        axum::serve(self.listener, self.router.into_make_service()).await
    }
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
