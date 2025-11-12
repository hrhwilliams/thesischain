use thesischain::app::App;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();

    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let port = std::env::var("PORT")
        .expect("Missing PORT")
        .parse()
        .expect("Invalid PORT");
    let listener = TcpListener::bind(("localhost", port)).await.expect("bind");

    tracing::info!(port = port, "Starting app");

    let app = App::new(listener);
    app.serve().await
}
