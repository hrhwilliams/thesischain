use axum::routing::{any, get, post};

use crate::AppState;

mod auth;
mod channel;
mod device;
// mod channel;
// mod key;
mod me;
mod ws;
// mod user;

pub struct Api;

impl Api {
    pub fn new() -> axum::Router<AppState> {
        axum::Router::new()
            .route("/auth/register", post(auth::register))
            .route("/auth/login", post(auth::login))
            .route("/auth/logout", post(auth::logout))
            .route("/auth/discord", get(auth::get_discord_oauth_url))
            .route("/auth/redirect", get(auth::discord_redirect))
            .route("/channel", post(channel::create_channel_with))
            .route(
                "/channel/{channel_id}",
                get(channel::get_channel_info), /* .post(channel::send_message) */
            )
            .route(
                "/channel/{channel_id}/history",
                get(channel::get_channel_history),
            )
            .route(
                "/channel/{channel_id}/{device_id}/otk",
                get(channel::get_user_device_otk),
            )
            .route("/me", get(me::me))
            .route("/me/channels", get(channel::get_all_channels))
            .route("/me/device", post(device::new_device))
            .route("/me/devices", get(device::get_devices))
            .route(
                "/me/device/{device_id}",
                get(device::get_device).post(device::upload_keys),
            )
            .route(
                "/me/device/{device_id}/otks",
                get(device::get_otks).post(device::upload_otks),
            )
            // .route("/user/{user_id}", get(device::get_user_info))
            // .route("/user/{user_id}/devices", get(device::get_user_devices))
            // .route(
            //     "/user/{user_id}/device/{device_id}",
            //     get(device::get_user_device),
            // )
            .route("/ws", any(ws::handle_websocket))
    }
}
