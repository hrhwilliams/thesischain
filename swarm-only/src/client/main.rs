use lib::Me;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let username = std::env::var("USER").expect("Missing USER");
    let me = Me::new(username);
}