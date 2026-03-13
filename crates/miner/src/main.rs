use std::sync::Arc;

use clap::{Parser, Subcommand};
use ed25519_dalek::{SigningKey, VerifyingKey};
use miner::http::MinerApi;
use miner::{Chain, GenesisDevice, MinerInfo, create_genesis, network::Node};
use rand::RngCore;
use tokio::net::TcpListener;
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
        /// Path to hex-encoded genesis block file
        #[arg(short, long)]
        genesis: String,

        /// Path to hex-encoded ed25519 signing keypair file (64 bytes)
        #[arg(short, long)]
        key: String,

        /// Hex-encoded ed25519 backend verifying key (32 bytes) for attestation verification
        #[arg(long)]
        backend_key: Option<String>,

        /// libp2p listen address
        #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/0")]
        listen: String,

        /// HTTP API bind address
        #[arg(long, default_value = "0.0.0.0:0")]
        http: String,

        /// Backend URL for miner registration and peer discovery
        #[arg(long)]
        backend_url: Option<String>,
    },
    /// Generate a genesis block with a new bootstrap identity
    Genesis {
        /// Hex-encoded ed25519 backend signing key (32 bytes) for attestation signing.
        /// If provided, genesis devices get proper attestations.
        #[arg(long)]
        backend_key: Option<String>,
    },
}

fn generate_keypair() -> (SigningKey, [u8; 32]) {
    let mut rng = rand::thread_rng();

    let mut ed25519_secret = [0u8; 32];
    rng.fill_bytes(&mut ed25519_secret);
    let signing_key = SigningKey::from_bytes(&ed25519_secret);

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

fn hex_decode(hex: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
        return Err("hex string has odd length".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(Into::into))
        .collect()
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

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Genesis { backend_key } => {
            let (bootstrap_key, x25519_public) = generate_keypair();

            let backend_signing_key = backend_key.map(|hex| {
                let bytes = hex_decode(&hex).expect("invalid hex for backend key");
                SigningKey::from_bytes(
                    bytes
                        .as_slice()
                        .try_into()
                        .expect("backend key must be 32 bytes"),
                )
            });

            let device = GenesisDevice {
                user_id: Uuid::now_v7(),
                device_id: Uuid::now_v7(),
                signing_key: bootstrap_key.clone(),
                x25519: x25519_public,
            };

            let genesis = create_genesis(
                &bootstrap_key,
                current_timestamp(),
                &[device],
                backend_signing_key.as_ref(),
            )?;

            let encoded = bincode::serde::encode_to_vec(&genesis, bincode::config::standard())?;
            let hex = hex_encode(&encoded);

            println!("Genesis block created.");
            println!(
                "Bootstrap verifying key: {}",
                hex_encode(&bootstrap_key.verifying_key().to_bytes())
            );
            println!(
                "Bootstrap signing keypair: {}",
                hex_encode(&bootstrap_key.to_keypair_bytes())
            );
            println!("Genesis block ({} bytes): {hex}", encoded.len());
        }
        Command::Run {
            genesis,
            key,
            backend_key,
            listen,
            http,
            backend_url,
        } => {
            // Load genesis block from file
            let genesis_hex =
                std::fs::read_to_string(&genesis).expect("failed to read genesis file");
            let genesis_bytes = hex_decode(&genesis_hex)?;
            let (genesis_block, _): (miner::Block, _) =
                bincode::serde::decode_from_slice(&genesis_bytes, bincode::config::standard())?;

            tracing::info!(bytes = genesis_bytes.len(), "loaded genesis block");

            // Load signing key from file
            let key_hex = std::fs::read_to_string(&key).expect("failed to read key file");
            let key_bytes = hex_decode(&key_hex)?;
            let signing_key = SigningKey::from_keypair_bytes(
                key_bytes
                    .as_slice()
                    .try_into()
                    .expect("key file must contain 64-byte ed25519 keypair"),
            )
            .expect("invalid ed25519 keypair");

            // Parse optional backend verifying key
            let backend_verifying_key = backend_key.map(|hex| {
                let bytes = hex_decode(&hex).expect("invalid hex for backend key");
                VerifyingKey::from_bytes(
                    bytes
                        .as_slice()
                        .try_into()
                        .expect("backend key must be 32 bytes"),
                )
                .expect("invalid ed25519 verifying key")
            });

            // Create chain and node
            let chain = Chain::new(genesis_block, backend_verifying_key)?;
            let chain = Arc::new(RwLock::new(chain));

            tracing::info!("chain initialized with genesis block");

            let (mut node, tx_sender) = Node::new(Arc::clone(&chain), signing_key)?;
            node.listen(&listen)?;

            let libp2p_addr = node.wait_for_listen_addr().await;
            let peer_id = node.peer_id();

            tracing::info!(%peer_id, %libp2p_addr, "P2P node listening");

            // Start HTTP API
            let http_listener = TcpListener::bind(&http).await?;
            let http_addr = http_listener.local_addr()?;
            tracing::info!(%http_addr, "HTTP API listening");

            let miner_api =
                MinerApi::integrated(Arc::clone(&chain), tx_sender, backend_verifying_key);

            // Register with backend and discover peers
            if let Some(ref backend) = backend_url {
                let client = reqwest::Client::new();

                let registration = MinerInfo {
                    http_addr: format!("http://{http_addr}"),
                    peer_id: peer_id.to_string(),
                    multiaddr: libp2p_addr.to_string(),
                };

                let resp = client
                    .post(format!("{backend}/api/miners/register"))
                    .json(&registration)
                    .send()
                    .await?;

                if resp.status().is_success() {
                    tracing::info!("registered with backend at {backend}");
                } else {
                    tracing::warn!(status = %resp.status(), "failed to register with backend");
                }

                // Discover existing peers
                let resp = client.get(format!("{backend}/api/miners")).send().await?;

                if resp.status().is_success() {
                    let peers: Vec<MinerInfo> = resp.json().await?;
                    for peer in &peers {
                        if peer.peer_id == peer_id.to_string() {
                            continue;
                        }
                        let addr: libp2p::Multiaddr = match peer.multiaddr.parse() {
                            Ok(a) => a,
                            Err(e) => {
                                tracing::warn!(addr = %peer.multiaddr, "invalid multiaddr: {e}");
                                continue;
                            }
                        };
                        let pid: libp2p::PeerId = match peer.peer_id.parse() {
                            Ok(p) => p,
                            Err(e) => {
                                tracing::warn!(id = %peer.peer_id, "invalid peer id: {e}");
                                continue;
                            }
                        };
                        if let Err(e) = node.dial(addr) {
                            tracing::warn!(%pid, "failed to dial peer: {e}");
                        } else {
                            node.add_explicit_peer(&pid);
                            tracing::info!(%pid, "dialed peer from backend registry");
                        }
                    }
                    tracing::info!(count = peers.len(), "discovered peers from backend");
                }
            }

            // Run P2P node and HTTP API concurrently
            tokio::select! {
                () = node.run(std::time::Duration::from_secs(5)) => {
                    tracing::warn!("P2P node exited");
                }
                result = miner_api.run(http_listener) => {
                    if let Err(e) = result {
                        tracing::error!("HTTP API error: {e}");
                    }
                }
            }
        }
    }

    Ok(())
}
