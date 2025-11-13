use core::identity::Identity;
use std::collections::HashMap;
use std::sync::Arc;
use tendermint_abci::ServerBuilder;
use tokio::sync::Mutex;

// This will be our in-memory "database"
// In a real app, this would be a connection to RocksDB or LMDB.
pub type AppState = Arc<Mutex<HashMap<String, String>>>;

// Import our application logic
mod app;
use crate::app::AbciApp;

#[tokio::main]
async fn main() -> Result<(), tendermint_abci::Error> {
    // 1. Create our application's state
    // We will have two "tables":
    //  - One for the DID registry
    //  - One for the name auction winners
    // let did_registry = Arc::new(Mutex::new(HashMap::new()));
    // let auction_winners = Arc::new(Mutex::new(HashMap::new()));
    let users = Arc::new(Mutex::new(HashMap::<String, Identity>::new()));

    let id = Identity::new("Maki".to_string());
    assert!(id.verify());

    // Create an instance of our application
    let app = AbciApp::new(users);

    // 2. Build and run the ABCI server
    let server = ServerBuilder::new(1024) // 1024 is buffer size
        .bind(("127.0.0.1", 0), app)
        .expect("Failed to start server");

    let addr = server.local_addr();
    println!("{}", addr);

    // 3. Wait for the server to stop (e.g., on Ctrl+C)
    server.listen()
}
