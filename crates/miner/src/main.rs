use std::sync::Arc;

use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;
use miner::{Chain, GenesisDevice, create_genesis, network::Node};
use rand::RngCore;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "miner", about = "ThesisChain blockchain key directory node")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start a blockchain node
    Run {
        /// Address to listen on (e.g. /ip4/0.0.0.0/tcp/0)
        #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/0")]
        listen: String,
    },
    /// Generate a genesis block with a new bootstrap identity
    Genesis,
}

fn generate_keypair() -> (SigningKey, [u8; 32]) {
    let mut rng = rand::thread_rng();

    let mut ed25519_secret = [0u8; 32];
    rng.fill_bytes(&mut ed25519_secret);
    let signing_key = SigningKey::from_bytes(&ed25519_secret);

    // Generate random x25519 public key bytes for the chain record.
    // In a real client, this would be a properly derived x25519 public key.
    let mut x25519_public = [0u8; 32];
    rng.fill_bytes(&mut x25519_public);

    (signing_key, x25519_public)
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_secs()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Genesis => {
            let (bootstrap_key, x25519_public) = generate_keypair();

            let device = GenesisDevice {
                user_id: Uuid::now_v7(),
                device_id: Uuid::now_v7(),
                signing_key: bootstrap_key.clone(),
                x25519: x25519_public,
            };

            let genesis = create_genesis(&bootstrap_key, current_timestamp(), &[device], None)?;

            let encoded = bincode::serde::encode_to_vec(&genesis, bincode::config::standard())?;
            let hex = hex_encode(&encoded);

            println!("Genesis block created.");
            println!(
                "Bootstrap ed25519 key: {}",
                hex_encode(&bootstrap_key.verifying_key().to_bytes())
            );
            println!(
                "Bootstrap signing key (secret): {}",
                hex_encode(&bootstrap_key.to_keypair_bytes())
            );
            println!("Genesis block ({} bytes): {hex}", encoded.len());
        }
        Command::Run { listen } => {
            let (signing_key, x25519_public) = generate_keypair();

            let device = GenesisDevice {
                user_id: Uuid::now_v7(),
                device_id: Uuid::now_v7(),
                signing_key: signing_key.clone(),
                x25519: x25519_public,
            };

            let genesis = create_genesis(&signing_key, current_timestamp(), &[device], None)?;
            let chain = Chain::new(genesis, None)?;

            tracing::info!("chain initialized with genesis block");

            let chain = Arc::new(RwLock::new(chain));
            let (mut node, _tx_sender) = Node::new(chain, signing_key)?;
            node.listen(&listen)?;

            tracing::info!("node started, entering event loop");
            node.run(std::time::Duration::from_secs(5)).await;
        }
    }

    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}
