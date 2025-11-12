use std::collections::HashMap;

use libp2p::{
    futures::StreamExt,
    identity,
    kad::{self, Mode, store::MemoryStore},
    mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use tokio::sync::{
    mpsc::Sender,
    oneshot,
};

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
}

enum SwarmCommand {
    PutRecord { key: String, value: String },
}

enum SwarmResponse {
    PutRecord(anyhow::Result<()>),
}

struct SwarmRequest {
    command: SwarmCommand,
    reply: oneshot::Sender<SwarmResponse>,
}

#[derive(Clone)]
pub struct EdgeNodes {
    tx: Sender<SwarmRequest>,
}

impl EdgeNodes {
    pub fn new() -> anyhow::Result<Self> {
        let local_key = identity::Keypair::generate_ed25519();

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_quic()
            .with_behaviour(|key| {
                Ok(Behaviour {
                    kademlia: kad::Behaviour::new(
                        key.public().to_peer_id(),
                        MemoryStore::new(key.public().to_peer_id()),
                    ),
                    mdns: mdns::tokio::Behaviour::new(
                        mdns::Config::default(),
                        key.public().to_peer_id(),
                    )?,
                })
            })?
            .build();

        swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));
        let (tx, mut rx) = tokio::sync::mpsc::channel::<SwarmRequest>(128);

        swarm.listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse()?)?;

        tokio::spawn(async move {
            let mut pending_put_requests: HashMap<kad::QueryId, oneshot::Sender<SwarmResponse>> =
                HashMap::new();

            loop {
                tokio::select! {
                    message = rx.recv() => {
                        match message {
                            Some(request) => match request.command {
                                SwarmCommand::PutRecord { key, value } => {
                                    let record = kad::Record {
                                        key: kad::RecordKey::new(&key),
                                        value: value.into(),
                                        publisher: None,
                                        expires: None,
                                    };

                                    match swarm
                                        .behaviour_mut()
                                        .kademlia
                                        .put_record(record, kad::Quorum::One)
                                    {
                                        Ok(query_id) => {
                                            pending_put_requests.insert(query_id, request.reply);
                                        }
                                        Err(err) => {
                                            let _ = request
                                                .reply
                                                .send(SwarmResponse::PutRecord(Err(err.into())));
                                        }
                                    }
                                }
                            },
                            None => break,
                        }
                    },
                    event = swarm.select_next_some() => {
                        match event {
                            SwarmEvent::NewListenAddr { .. } => {
                            },
                            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                                for (peer_id, multiaddr) in list {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                                }
                            },
                            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                                kad::Event::OutboundQueryProgressed { id, result, .. },
                            )) => {
                                if let kad::QueryResult::PutRecord(outcome) = result {
                                    let response = match outcome {
                                        Ok(kad::PutRecordOk { key }) => {
                                            println!(
                                                "Successfully put record {:?}",
                                                std::str::from_utf8(key.as_ref()).unwrap()
                                            );
                                            Ok(())
                                        }
                                        Err(err) => {
                                            eprintln!("Failed to put record: {err:?}");
                                            Err(err.into())
                                        }
                                    };

                                    if let Some(reply) = pending_put_requests.remove(&id) {
                                        let _ = reply.send(SwarmResponse::PutRecord(response));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    Ok(Self { tx })
    }

    pub async fn put(&self, key: String, value: String) -> anyhow::Result<()> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let request = SwarmRequest {
            command: SwarmCommand::PutRecord { key, value },
            reply: reply_tx,
        };

        self.tx
            .send(request)
            .await
            .map_err(anyhow::Error::new)?;

        match reply_rx.await.map_err(anyhow::Error::new)? {
            SwarmResponse::PutRecord(result) => result,
        }
    }
}
