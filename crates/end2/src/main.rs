use std::sync::Arc;

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use diesel::{PgConnection, r2d2::ConnectionManager};
use ed25519_dalek::SigningKey;
use end2::{
    App, AppState, ChainDeviceKeyService, DbAuthService, DbDeviceKeyService, DbMessageRelayService,
    DbOtkService, OAuthHandler,
};
use mimalloc::MiMalloc;
use tokio::net::TcpListener;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // IP and port to run server on
    let ip = std::env::var("BACKEND_IP").expect("BACKEND_IP must be set");
    let port: u16 = std::env::var("BACKEND_PORT")
        .expect("BACKEND_PORT must be set")
        .parse()
        .expect("BACKEND_PORT must be in range 0-65535");

    // Load signing key that the backend uses to sign keys distributed by the blockchain
    // so that in a way it acts as a CA
    let signing_key = std::env::var("SERVER_SIGNING_KEY").expect("SERVER_SIGNING_KEY must be set");
    let signing_key_bytes = BASE64_STANDARD_NO_PAD
        .decode(signing_key)
        .expect("invalid base64");
    let signing_key = SigningKey::from_bytes(
        signing_key_bytes
            .as_slice()
            .try_into()
            .expect("invalid key length"),
    );

    // Load variables for Discord OAuth2 flow
    let client_id =
        std::env::var("DISCORD_OAUTH_CLIENT_ID").expect("DISCORD_OAUTH_CLIENT_ID must be set");
    let client_secret =
        std::env::var("DISCORD_OAUTH_SECRET").expect("DISCORD_OAUTH_SECRET must be set");
    let redirect =
        std::env::var("DISCORD_OAUTH_REDIRECT").expect("DISCORD_OAUTH_REDIRECT must be set");
    let oauth = OAuthHandler::new(client_id, client_secret, redirect);

    // Load PostgreSQL database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to connect to Postgres");

    // Use blockchain-based key distribution or single entity key distribution
    let chain_mode = std::env::var("CHAIN_MODE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let app = if chain_mode {
        tracing::info!("starting in blockchain mode");

        let auth = Arc::new(DbAuthService::new(pool.clone()));
        let device_keys = Arc::new(ChainDeviceKeyService::new());
        let otks = Arc::new(DbOtkService::new(pool.clone()));
        let relay = Arc::new(DbMessageRelayService::new(pool.clone()));
        let app_state = AppState::new(auth, device_keys, otks, relay, oauth, pool, signing_key);

        App::from_state(app_state)
    } else {
        tracing::info!("starting in centralized mode");

        let auth = Arc::new(DbAuthService::new(pool.clone()));
        let device_keys = Arc::new(DbDeviceKeyService::new(pool.clone()));
        let otks = Arc::new(DbOtkService::new(pool.clone()));
        let relay = Arc::new(DbMessageRelayService::new(pool.clone()));
        let app_state = AppState::new(auth, device_keys, otks, relay, oauth, pool, signing_key);

        App::from_state(app_state)
    };

    let listener = TcpListener::bind((ip, port)).await.expect("TcpListener");

    app.run(listener).await
}
