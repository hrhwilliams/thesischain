use libp2p::kad::store::MemoryStore;
use libp2p::{
    gossipsub, identity, kad, mdns, noise, quic,
    swarm::{NetworkBehaviour, StreamProtocol, SwarmEvent},
    yamux,
};
use tracing_subscriber::EnvFilter;

const IPFS_PROTO_NAME: StreamProtocol = StreamProtocol::new("/ipfs/kad/1.0.0");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    // Create a random key for ourselves.
    let local_key = identity::Keypair::generate_ed25519();

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_quic()
        .with_dns()?
        .with_behaviour(|key| {
            // Create a Kademlia behaviour.
            let mut cfg = kad::Config::new(IPFS_PROTO_NAME);
            cfg.set_query_timeout(std::time::Duration::from_secs(5 * 60));
            let store = kad::store::MemoryStore::new(key.public().to_peer_id());
            kad::Behaviour::with_config(key.public().to_peer_id(), store, cfg)
        })?
        .build();

    let mut pk_record_key = vec![];
    pk_record_key.put_slice("/pk/".as_bytes());
    pk_record_key.put_slice(swarm.local_peer_id().to_bytes().as_slice());

    let mut pk_record = kad::Record::new(pk_record_key, local_key.public().encode_protobuf());
    pk_record.publisher = Some(*swarm.local_peer_id());
    pk_record.expires = Some(Instant::now().add(Duration::from_secs(60)));

    swarm
        .behaviour_mut()
        .put_record(pk_record, kad::Quorum::N(NonZeroUsize::new(3).unwrap()))?;

    Ok(())
}
