use axum::http::Method;
use axum::http::header;
use axum::middleware;
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::Api;
use crate::AppState;
use crate::OAuthHandler;
use crate::session::create_session;

/// flow
/// - client registers username and x25519/ed25519 key bundle
/// - client can do challenge-response to get a session token

/// # API map
///
/// |verb|endpoint|description|
/// |----|--------|-----------|
/// |GET|`/api/auth/me`|returns struct containing info about the current user
/// |POST|`/api/auth/register`|creates a new account
/// |GET|`/api/auth/challenge`|request challenge string from server
/// |POST|`/api/auth/challenge`|prove ownership of keys from challenge string, returns session token
/// |GET|`/api/keys/`|returns struct containing user's identity keys and active one-time keys
/// |POST|`/api/keys/id`|user uploads ed25519 and x25519 keys. fails if user already has existing keys
/// |PUT|`/api/keys/id`|user uploads new ed25519 and x25519 keys signed with previous ed25519 key
/// |POST|`/api/keys/otk`|user uploads bundle of one-time keys
/// |GET|`/api/usr/{user_id}`|request information about a user, including their public keys
/// |GET|`/api/rooms`|get all rooms a user is a participant in
/// |POST|`/api/room/{user_id}`|create a room between self and {user_id}, returns a room ID
pub struct App {
    router: axum::Router,
}

impl App {
    pub fn new(oauth: OAuthHandler, pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        let app_state = AppState::new(oauth, pool);
        // let router = axum::Router::new()
        //     .route("/", get(web_endpoints::index))
        //     .route(
        //         "/login",
        //         get(web_endpoints::display_login_form).post(web_endpoints::login),
        //     )
        //     .route("/logout", get(web_endpoints::logout))
        //     .route(
        //         "/register",
        //         get(web_endpoints::register_form).post(web_endpoints::register),
        //     )
        //     .route("/dms", get(web_endpoints::direct_messages))
        //     .route("/dm", post(web_endpoints::create_room))
        //     .route("/dm/{room_id}", get(web_endpoints::direct_message))
        //     .route("/dm/ws/{room_id}", any(web_endpoints::direct_message_ws))
        //     .layer(TraceLayer::new_for_http())
        //     .with_state(state);

        let router = axum::Router::new()
            .nest("/api", Api::new())
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                create_session,
            ))
            .layer(
                CorsLayer::new()
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::OPTIONS])
                    .allow_headers([header::CONTENT_TYPE, header::ACCEPT])
                    .allow_origin([
                        "http://127.0.0.1:8080".parse().unwrap(),
                        "http://localhost:8080".parse().unwrap(),
                    ])
                    .allow_credentials(true),
            )
            .layer(TraceLayer::new_for_http())
            .with_state(app_state);

        Self { router }
    }

    pub async fn run(self, listener: TcpListener) -> Result<(), std::io::Error> {
        axum::serve(listener, self.router).await
    }
}
