use std::collections::HashMap;
use std::sync::Arc;

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use diesel::{PgConnection, r2d2::ConnectionManager};
use ed25519_dalek::SigningKey;
use end2::{
    App, AppState, AuthService, CookieWebSessionService, DbAuthService, DbDeviceKeyService,
    DbMessageRelayService, DbOtkService, DeviceKeyService, MessageRelayService, OAuthHandler,
    OAuthInfo, OtkService, WebSessionService,
};
use mimalloc::MiMalloc;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::metrics::PeriodicReader;
use r2d2::Pool;
use tokio::net::TcpListener;
use tracing_subscriber::prelude::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const DISCORD_AUTHORIZE_URL: &str = "https://discord.com/oauth2/authorize";
const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";

// see https://developers.google.com/identity/openid-connect/openid-connect
const GOOGLE_AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

fn init_telemetry() {
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("span exporter");
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .build();

    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .build()
        .expect("log exporter");
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .build();

    let tracer = tracer_provider.tracer("end2");
    let otel_trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let otel_logs_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .with(otel_trace_layer)
        .with(otel_logs_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .build()
        .expect("metric exporter");
    let reader = PeriodicReader::builder(metric_exporter).build();
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .build();
    opentelemetry::global::set_meter_provider(meter_provider.clone());
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();

    init_telemetry();

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
    let discord_oauth_info = OAuthInfo {
        client_id,
        client_secret,
        redirect,
        auth_url: DISCORD_AUTHORIZE_URL.to_string(),
        token_url: DISCORD_TOKEN_URL.to_string(),
        scopes: vec!["identify"],
    };
    let discord_oauth = OAuthHandler::new(discord_oauth_info);

    // Load variables for Google OAuth2 flow
    let client_id =
        std::env::var("GOOGLE_OAUTH_CLIENT_ID").expect("GOOGLE_OAUTH_CLIENT_ID must be set");
    let client_secret =
        std::env::var("GOOGLE_OAUTH_SECRET").expect("GOOGLE_OAUTH_SECRET must be set");
    let redirect =
        std::env::var("GOOGLE_OAUTH_REDIRECT").expect("GOOGLE_OAUTH_REDIRECT must be set");
    let google_oauth_info = OAuthInfo {
        client_id,
        client_secret,
        redirect,
        auth_url: GOOGLE_AUTHORIZE_URL.to_string(),
        token_url: GOOGLE_TOKEN_URL.to_string(),
        scopes: vec!["openid", "email", "profile"],
    };
    let google_oauth = OAuthHandler::new(google_oauth_info);

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
        unimplemented!();

        // let auth = Arc::new(DbAuthService::new(pool.clone()));
        // let device_keys = Arc::new(ChainDeviceKeyService::new());
        // let otks = Arc::new(DbOtkService::new(pool.clone()));
        // let relay = Arc::new(DbMessageRelayService::new(pool.clone()));
        // let app_state = AppState::new(auth, device_keys, otks, relay, oauth, pool, signing_key);

        // App::new(app_state)
    } else {
        tracing::info!("starting in centralized mode");

        let app_state = AppStateBuilder::new(pool, signing_key)
            .auth_service(DbAuthService::new)
            .key_directory(DbDeviceKeyService::new)
            .otk_directory(DbOtkService::new)
            .message_relay(DbMessageRelayService::new)
            .web_session_service(CookieWebSessionService::new)
            .with_oauth_handler("google", google_oauth)
            .with_oauth_handler("discord", discord_oauth)
            .build();

        App::new(app_state)
    };

    let listener = TcpListener::bind((ip, port)).await.expect("TcpListener");

    app.run(listener).await
}

struct AppStateBuilder {
    pool: Pool<ConnectionManager<PgConnection>>,
    signing_key: SigningKey,
    auth_constructor: Option<
        fn(
            Pool<ConnectionManager<PgConnection>>,
            HashMap<String, OAuthHandler>,
        ) -> impl AuthService,
    >,
    key: Option<impl DeviceKeyService>,
    otk: Option<impl OtkService>,
    relay: Option<impl MessageRelayService>,
    web_sessions: Option<impl WebSessionService>,
    oauth: HashMap<String, OAuthHandler>,
}

impl AppStateBuilder {
    fn new(pool: Pool<ConnectionManager<PgConnection>>, signing_key: SigningKey) -> Self {
        Self {
            pool,
            signing_key,
            auth_constructor: None,
            key: None,
            otk: None,
            relay: None,
            web_sessions: None,
            oauth: HashMap::new(),
        }
    }

    fn auth_service<T: AuthService + 'static>(
        mut self,
        constructor: fn(Pool<ConnectionManager<PgConnection>>, HashMap<String, OAuthHandler>) -> T,
    ) -> Self {
        self.auth_constructor = Some(constructor);
        self
    }

    fn key_directory<T: DeviceKeyService + 'static>(
        mut self,
        constructor: fn(Pool<ConnectionManager<PgConnection>>) -> T,
    ) -> Self {
        self.key = Some(constructor(self.pool.clone()));
        self
    }

    fn otk_directory<T: OtkService + 'static>(
        mut self,
        constructor: fn(Pool<ConnectionManager<PgConnection>>) -> T,
    ) -> Self {
        self.otk = Some(constructor(self.pool.clone()));
        self
    }

    fn message_relay<T: MessageRelayService + 'static>(
        mut self,
        constructor: fn(Pool<ConnectionManager<PgConnection>>) -> T,
    ) -> Self {
        self.relay = Some(constructor(self.pool.clone()));
        self
    }

    fn web_session_service<W: WebSessionService + 'static>(
        mut self,
        constructor: fn(Pool<ConnectionManager<PgConnection>>) -> W,
    ) -> Self {
        self.web_sessions = Some(constructor(self.pool.clone()));
        self
    }

    fn with_oauth_handler(mut self, name: impl Into<String>, handler: OAuthHandler) -> Self {
        self.oauth.insert(name.into(), handler);
        self
    }

    fn build(self) -> AppState {
        AppState::new(
            self.auth_constructor.expect("auth service not set")(self.pool.clone(), self.oauth),
            self.key.expect("key directory not set"),
            self.otk.expect("otk directory not set"),
            self.relay.expect("message relay not set"),
            self.web_sessions.expect("web session service not set"),
            self.pool,
            self.signing_key,
        )
    }
}
