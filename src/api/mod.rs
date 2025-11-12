use axum::{Router, routing::post};

use crate::app::AppState;

mod errors;
mod routes;

pub struct ApiRoutes;

impl ApiRoutes {
    pub fn router() -> Router<AppState> {
        Router::new().route("/register", post(routes::register))
    }
}
