use axum::routing::{get, post};

use crate::AppState;

mod auth;
mod key;
mod room;
mod user;

pub struct Api;

impl Api {
    pub fn new() -> axum::Router<AppState> {
        axum::Router::new()
            .route("/auth/me", get(auth::me))
            .route("/auth/register", post(auth::register))
            .route(
                "/auth/challenge",
                get(auth::get_challenge).post(auth::post_challenge),
            )
    }
}
