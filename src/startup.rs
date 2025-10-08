use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use std::collections::{HashMap, HashSet};
use std::net::TcpListener;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::chain::{Node, NodeContent};
use crate::peer::Peer;
use crate::routes;
use crate::routes::{BlockMessage, PeerList};

pub struct KeyChain {
    server: Server,
    port: u16,
}

pub struct KeyChainData {
    port: u16,
    pub peers: RwLock<HashSet<Peer>>,
    pub chain: RwLock<Vec<Node<NodeContent>>>,
    pub health_check_interval: Duration,
    pub gossip_interval: Duration,
}

impl KeyChain {
    pub async fn build() -> anyhow::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();

        let genesis_node = tokio::task::spawn_blocking(|| Node::new("genesis".to_string())).await?;

        let keychain = web::Data::new(KeyChainData {
            port,
            peers: RwLock::new(HashSet::new()),
            chain: RwLock::new(vec![genesis_node]),
            health_check_interval: Duration::from_secs(5),
            gossip_interval: Duration::from_secs(5),
        });

        let data = keychain.clone();

        let server = HttpServer::new(move || {
            App::new()
                // .wrap(Logger::default())
                .app_data(keychain.clone())
                .route("/", web::get().to(routes::index))
                .route("/add_peer", web::post().to(routes::add_peer)) // GET /add_peer
                .route("/add_peer", web::get().to(routes::add_peer_form)) // GET /add_peer
                .route("/add_peer_manual", web::post().to(routes::add_peer_manual))
                .route("/get_peers", web::get().to(routes::get_peers))
                .route("/chain", web::get().to(routes::get_chain))
                .route("/block", web::post().to(routes::add_block))
                .route("/health_check", web::get().to(routes::health_check))
        })
        .listen(listener)?
        .workers(1)
        .run();

        tokio::spawn(health_check_task(data.clone()));
        tokio::spawn(gossip_task(data.clone()));
        tokio::spawn(mining_task(data.clone()));

        Ok(Self { server, port })
    }

    pub async fn run(self) -> anyhow::Result<(), std::io::Error> {
        self.server.await
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl KeyChainData {
    pub async fn insert_peers(&self, peer_list: Vec<Peer>) {
        if !peer_list.is_empty() {
            let mut peers = self.peers.write().await;

            for peer in peer_list {
                let address = peer.address.clone();
                if !peer.address.contains(&format!(":{}", self.port)) && peers.insert(peer) {
                    log::info!("Added peer {}", address);
                }
            }
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

async fn health_check_task(app: web::Data<KeyChainData>) -> anyhow::Result<()> {
    loop {
        let peers = {
            let peers = app.peers.read().await;
            peers.clone()
        };

        let remove_list = {
            let mut remove_list = vec![];
            let client = reqwest::Client::new();

            for peer in peers.iter() {
                match client
                    .get(format!("{}/health_check", peer.address))
                    .timeout(Duration::from_secs(3))
                    .send()
                    .await
                {
                    Ok(response) if response.status().is_success() => {
                        log::trace!("Peer {} is healthy", peer.address);
                    }
                    Ok(response) => {
                        log::warn!("Got response {} from {}", response.status(), peer.address);
                        remove_list.push(peer);
                    }
                    Err(e) => {
                        log::warn!("{}", e);
                        remove_list.push(peer);
                    }
                }
            }

            remove_list
        };

        if !remove_list.is_empty() {
            let mut peers = app.peers.write().await;

            for peer in remove_list {
                peers.remove(peer);
            }
        }

        tokio::time::sleep(app.health_check_interval).await;
    }
}

async fn gossip_task(app: web::Data<KeyChainData>) -> anyhow::Result<()> {
    loop {
        let peers = {
            let peers = app.peers.read().await;
            peers.clone()
        };

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "content-type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let mut peers_to_add: Vec<Peer> = vec![];
        let mut all_chains: HashMap<Peer, Vec<Node<NodeContent>>> = HashMap::new();

        for peer in peers.iter() {
            match client
                .get(format!("{}/get_peers", peer.address))
                .timeout(Duration::from_secs(3))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    if let Ok(new_peers) = serde_json::from_str::<PeerList>(&response.text().await?)
                    {
                        for peer in new_peers.peers {
                            if !peers.contains(&peer) {
                                peers_to_add.push(peer);
                            }
                        }
                    } else {
                        log::warn!("Got invalid response from {}", peer.address);
                    }
                }
                _ => {
                    log::warn!("Failed to gossip peers from {}", peer.address);
                }
            }

            match client
                .get(format!("{}/chain", peer.address))
                .timeout(Duration::from_secs(3))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    if let Ok(chain) = response.json::<Vec<Node<NodeContent>>>().await {
                        all_chains.insert(peer.clone(), chain);
                    } else {
                        log::warn!("Got invalid chain from {}", peer.address);
                    }
                }
                _ => {
                    log::warn!("Failed to get chain from {}", peer.address);
                }
            }
        }

        if let Some(longest_chain) = all_chains.values().max_by_key(|c| c.len()) {
            let mut local_chain = app.chain.write().await;
            if longest_chain.len() > local_chain.len() {
                log::info!("Found longer chain, updating local chain.");
                *local_chain = longest_chain.clone();
            }
        }

        app.get_ref().insert_peers(peers_to_add).await;
        tokio::time::sleep(app.gossip_interval).await;
    }
}

async fn mining_task(app: web::Data<KeyChainData>) -> anyhow::Result<()> {
    loop {
        let random_delay = rand::random::<u64>() % 20 + 5;
        tokio::time::sleep(Duration::from_secs(random_delay)).await;

        let parent = {
            let chain = app.chain.read().await;
            chain.last().unwrap().clone()
        };

        let message = format!("Block mined by {}", app.port());

        let new_block =
            tokio::task::spawn_blocking(move || Node::append(message, &parent)).await?;

        let peers = {
            let peers = app.peers.read().await;
            peers.clone()
        };

        let client = reqwest::Client::new();
        for peer in peers.iter() {
            let _ = client
                .post(format!("{}/block", peer.address))
                .json(&BlockMessage {
                    block: new_block.clone(),
                })
                .send()
                .await;
        }

        let mut chain = app.chain.write().await;
        if new_block.parent == Some(chain.last().unwrap().hash) {
            log::info!("Mined a new block and added to local chain.");
            chain.push(new_block);
        } else {
            log::info!("Mined block is outdated, starting over.");
        }
    }
}
