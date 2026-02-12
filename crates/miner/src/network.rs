use std::sync::Arc;
use std::time::Duration;

use ed25519_dalek::SigningKey;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{Swarm, SwarmBuilder, gossipsub, mdns, swarm::NetworkBehaviour};
use tokio::sync::{RwLock, mpsc};

use crate::chain::Chain;
use crate::crypto;
use crate::error::ChainError;
use crate::types::{Block, SignedTransaction};

const BLOCKS_TOPIC: &str = "thesischain/blocks/1.0.0";
const TXPOOL_TOPIC: &str = "thesischain/txpool/1.0.0";

#[derive(NetworkBehaviour)]
struct ChainBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

/// A P2P node that participates in the blockchain network.
pub struct Node {
    swarm: Swarm<ChainBehaviour>,
    chain: Arc<RwLock<Chain>>,
    pending_txs: Vec<SignedTransaction>,
    signing_key: SigningKey,
    blocks_topic: gossipsub::IdentTopic,
    txpool_topic: gossipsub::IdentTopic,
    tx_receiver: mpsc::Receiver<SignedTransaction>,
}

impl Node {
    /// Create a new node with the given chain and signing key.
    ///
    /// Returns the node and a sender channel for submitting transactions
    /// from the HTTP API.
    pub fn new(
        chain: Arc<RwLock<Chain>>,
        signing_key: SigningKey,
    ) -> Result<(Self, mpsc::Sender<SignedTransaction>), Box<dyn std::error::Error>> {
        let blocks_topic = gossipsub::IdentTopic::new(BLOCKS_TOPIC);
        let txpool_topic = gossipsub::IdentTopic::new(TXPOOL_TOPIC);
        let (tx_sender, tx_receiver) = mpsc::channel(256);

        let swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_behaviour(|key| {
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(1))
                    .build()
                    .expect("valid gossipsub config");

                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )
                .expect("valid gossipsub behaviour");

                let mdns =
                    mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())
                        .expect("valid mdns behaviour");

                ChainBehaviour { gossipsub, mdns }
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        Ok((
            Self {
                swarm,
                chain,
                pending_txs: Vec::new(),
                signing_key,
                blocks_topic,
                txpool_topic,
                tx_receiver,
            },
            tx_sender,
        ))
    }

    /// Returns a reference to the shared chain.
    #[must_use]
    pub const fn chain(&self) -> &Arc<RwLock<Chain>> {
        &self.chain
    }

    /// Start listening on the given address and subscribe to topics.
    pub fn listen(&mut self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let multiaddr: libp2p::Multiaddr = addr.parse()?;
        self.swarm.listen_on(multiaddr)?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.blocks_topic)?;
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.txpool_topic)?;

        Ok(())
    }

    /// Get the peer ID of this node.
    #[must_use]
    pub fn peer_id(&self) -> libp2p::PeerId {
        *self.swarm.local_peer_id()
    }

    /// Return the first listen address (available after `listen()`).
    ///
    /// Returns `None` if the swarm hasn't started listening yet.
    #[must_use]
    pub fn listen_addr(&self) -> Option<libp2p::Multiaddr> {
        self.swarm.listeners().next().cloned()
    }

    /// Dial a peer at the given multiaddr and add them as an explicit gossipsub peer.
    pub fn dial(&mut self, addr: libp2p::Multiaddr) -> Result<(), Box<dyn std::error::Error>> {
        self.swarm.dial(addr)?;
        Ok(())
    }

    /// Add a peer as an explicit gossipsub peer (they will always be in the mesh).
    pub fn add_explicit_peer(&mut self, peer_id: &libp2p::PeerId) {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .add_explicit_peer(peer_id);
    }

    /// Submit a transaction to the local mempool and gossip it to peers.
    pub fn submit_transaction(&mut self, tx: SignedTransaction) -> Result<(), ChainError> {
        crypto::verify_transaction(&tx)?;

        let encoded = bincode::serde::encode_to_vec(&tx, bincode::config::standard())
            .map_err(|e| ChainError::SerializationError(e.to_string()))?;

        self.pending_txs.push(tx);

        let _ = self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.txpool_topic.clone(), encoded);

        Ok(())
    }

    /// Wait for the swarm to resolve a listen address (must call after `listen()`).
    ///
    /// Returns the first `NewListenAddr` multiaddr.
    pub async fn wait_for_listen_addr(&mut self) -> libp2p::Multiaddr {
        loop {
            let event = self.swarm.select_next_some().await;
            if let SwarmEvent::NewListenAddr { address, .. } = event {
                tracing::info!(%address, "listening on");
                return address;
            }
        }
    }

    /// Run the node event loop. This drives the libp2p swarm and handles
    /// incoming blocks, transactions, and peer discovery.
    pub async fn run(&mut self, block_interval: Duration) {
        let mut block_timer = tokio::time::interval(block_interval);

        loop {
            tokio::select! {
                _ = block_timer.tick() => {
                    self.try_produce_block().await;
                }
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                }
                Some(tx) = self.tx_receiver.recv() => {
                    if let Err(e) = self.submit_transaction(tx) {
                        tracing::warn!("rejected submitted tx: {e}");
                    }
                }
            }
        }
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<ChainBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(ChainBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                message,
                ..
            })) => {
                self.handle_gossip_message(&message).await;
            }
            SwarmEvent::Behaviour(ChainBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                for (peer_id, addr) in peers {
                    self.swarm.add_peer_address(peer_id, addr);
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                    tracing::info!(%peer_id, "discovered peer via mDNS");
                }
            }
            SwarmEvent::Behaviour(ChainBehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                for (peer_id, _addr) in peers {
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                    tracing::info!(%peer_id, "peer expired from mDNS");
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!(%address, "listening on");
            }
            _ => {}
        }
    }

    async fn handle_gossip_message(&mut self, message: &gossipsub::Message) {
        let topic = message.topic.as_str();

        if topic == BLOCKS_TOPIC {
            match bincode::serde::decode_from_slice::<Block, _>(
                &message.data,
                bincode::config::standard(),
            ) {
                Ok((block, _)) => {
                    let mut chain = self.chain.write().await;
                    match chain.append(block) {
                        Ok(()) => {
                            tracing::info!(height = chain.height(), "appended new block from peer");
                            // Prune confirmed txs using the chain state
                            self.pending_txs.retain(|tx| {
                                chain.state().verify_nonce(&tx.signer, tx.nonce).is_ok()
                            });
                        }
                        Err(e) => {
                            tracing::warn!("rejected block from peer: {e}");
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to deserialize block: {e}");
                }
            }
        } else if topic == TXPOOL_TOPIC {
            match bincode::serde::decode_from_slice::<SignedTransaction, _>(
                &message.data,
                bincode::config::standard(),
            ) {
                Ok((tx, _)) => {
                    if crypto::verify_transaction(&tx).is_ok() {
                        tracing::debug!(signer = ?tx.signer, "received transaction from peer");
                        self.pending_txs.push(tx);
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to deserialize transaction: {e}");
                }
            }
        }
    }

    async fn try_produce_block(&mut self) {
        if self.pending_txs.is_empty() {
            return;
        }

        let chain = self.chain.read().await;
        let author_key = self.signing_key.verifying_key().to_bytes();

        if !chain.state().is_authority(&author_key) {
            return;
        }

        let previous_hash = match chain.head_hash() {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("failed to compute tip hash: {e}");
                return;
            }
        };

        let index = chain.height();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs();

        // Take all pending transactions
        let txs = std::mem::take(&mut self.pending_txs);

        drop(chain); // Release read lock before acquiring write lock

        let block =
            match crypto::sign_block(index, timestamp, previous_hash, txs, &self.signing_key) {
                Ok(b) => b,
                Err(e) => {
                    tracing::error!("failed to sign block: {e}");
                    return;
                }
            };

        // Broadcast the block
        let encoded = match bincode::serde::encode_to_vec(&block, bincode::config::standard()) {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("failed to serialize block: {e}");
                return;
            }
        };

        let _ = self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.blocks_topic.clone(), encoded);

        // Append to our own chain
        let mut chain = self.chain.write().await;
        match chain.append(block) {
            Ok(()) => {
                tracing::info!(height = chain.height(), "produced and appended new block");
            }
            Err(e) => {
                tracing::error!("failed to append own block: {e}");
            }
        }
    }
}
