use client::Client;
use defs::Identity;
use ed25519_dalek::SigningKey;

const BOOTSTRAP_URLS: &[&str; 3] = &[
    "http://localhost:10881",
    "http://localhost:10882",
    "http://localhost:10883",
];

fn random_username() -> String {
    todo!()
}

fn random_key() -> SigningKey {
    todo!()
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();

    let username = std::env::var("USERNAME").unwrap_or(random_username());
    let signing_key = std::env::var("SIGNING_KEY").unwrap_or(random_key());
    let identity = Identity::new(username, signing_key).expect("identity");

    let client = Client::new(identity)
        .try_connect(BOOTSTRAP_URLS)
        .await
        .run()
        .await;

    Ok(())
}
