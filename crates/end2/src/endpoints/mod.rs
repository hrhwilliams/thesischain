use axum::routing::{any, get, post};

use crate::AppState;

mod auth;
mod channel;
mod key;
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
            .route("/auth/logout", post(auth::logout))
            .route("/keys/{receiver}/id", get(key::get_identity_key))
            .route("/keys/{receiver}/otk", get(key::get_otk))
            .route("/keys/otk", get(key::count_otks).post(key::publish_otks))
            .route("/channels", get(channel::get_all_channels))
            .route("/channel/{receiver}", post(channel::create_channel_with))
            .route(
                "/channels/{channel_id}/userinfo",
                get(channel::get_channel_participant_info),
            )
            .route("/channel/ws/{channel_id}", any(channel::handle_websocket))
    }
}
