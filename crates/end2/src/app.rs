use axum::routing::any;
use axum::routing::get;
use axum::routing::post;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::AppState;
use crate::endpoints;

pub struct App {
    router: axum::Router,
}

impl App {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        let state = AppState::new(pool);
        let router = axum::Router::new()
            .route("/", get(endpoints::index))
            .route(
                "/login",
                get(endpoints::display_login_form).post(endpoints::login),
            )
            .route("/logout", get(endpoints::logout))
            .route(
                "/register",
                get(endpoints::register_form).post(endpoints::register),
            )
            .route("/dms", get(endpoints::direct_messages))
            // .route("/dm", post(endpoints::message_request))
            // .route("/dm/{room_id}", get(endpoints::direct_message))
            // .route("/dm/ws/{room_id}", any(endpoints::direct_message_ws))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        Self { router }
    }

    pub async fn run(self, listener: TcpListener) -> Result<(), std::io::Error> {
        axum::serve(listener, self.router).await
    }
}
