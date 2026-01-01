use end2::App;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let listener = TcpListener::bind(("127.0.0.1", 8080))
        .await
        .expect("TcpListener");

    let app = App::new();
    app.run(listener).await
}
