use diesel::{PgConnection, r2d2::ConnectionManager};
use end2::{App, OAuthHandler};
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

    let ip = std::env::var("BACKEND_IP").expect("BACKEND_IP must be set");

    let port = std::env::var("BACKEND_PORT")
        .expect("BACKEND_PORT must be set")
        .parse()
        .expect("BACKEND_PORT must be in range 0-65535");

    let client_id =
        std::env::var("DISCORD_OAUTH_CLIENT_ID").expect("DISCORD_OAUTH_CLIENT_ID must be set");
    let client_secret =
        std::env::var("DISCORD_OAUTH_SECRET").expect("DISCORD_OAUTH_SECRET must be set");
    let redirect =
        std::env::var("DISCORD_OAUTH_REDIRECT").expect("DISCORD_OAUTH_REDIRECT must be set");

    let oauth = OAuthHandler::new(client_id, client_secret, redirect);

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to connect to Postgres");

    let app = App::new(oauth, pool);

    let listener = TcpListener::bind((ip, port)).await.expect("TcpListener");

    app.run(listener).await
}
