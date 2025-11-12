use std::{collections::HashMap, time::Duration};

use libp2p::{
    StreamProtocol, Swarm,
    futures::StreamExt,
    identity,
    kad::{self, Mode, store::MemoryStore},
    mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use tokio::sync::{mpsc::Sender, oneshot};

mod swarm;

const IPFS_PROTO_NAME: StreamProtocol = StreamProtocol::new("/ipfs/kad/1.0.0");

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
}

enum SwarmCommand {
    PutRecord { key: String, value: String },
    GetRecord { key: String },
}

enum SwarmResponse {
    PutRecord(anyhow::Result<()>),
    GetRecord(anyhow::Result<Option<String>>),
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
            // .with_tcp(
            //     tcp::Config::default(),
            //     noise::Config::new,
            //     yamux::Config::default,
            // )?
            .with_quic()
            // .with_dns()?
            .with_behaviour(|key| {
                let mut cfg = kad::Config::new(IPFS_PROTO_NAME);
                cfg.set_query_timeout(Duration::from_secs(5 * 60));
                let store = kad::store::MemoryStore::new(key.public().to_peer_id());

                Ok(Behaviour {
                    kademlia: kad::Behaviour::with_config(key.public().to_peer_id(), store, cfg),
                    mdns: mdns::tokio::Behaviour::new(
                        mdns::Config::default(),
                        key.public().to_peer_id(),
                    )?,
                })
            })?
            .build();

        swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));
        let (tx, mut rx) = tokio::sync::mpsc::channel::<SwarmRequest>(128);

        // swarm.listen_on("/ip4/172.28.16.1/tcp/0".parse()?)?;
        swarm.listen_on("/ip4/172.28.16.1/udp/0/quic-v1".parse()?)?;

        tokio::spawn(async move {
            let mut pending_requests: HashMap<kad::QueryId, oneshot::Sender<SwarmResponse>> =
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
                                            pending_requests.insert(query_id, request.reply);
                                        }
                                        Err(err) => {
                                            let _ = request
                                                .reply
                                                .send(SwarmResponse::PutRecord(Err(err.into())));
                                        }
                                    }
                                },
                                SwarmCommand::GetRecord { key } => {
                                    let query_id = swarm
                                        .behaviour_mut()
                                        .kademlia
                                        .get_record(kad::RecordKey::new(&key));
                                    pending_requests.insert(query_id, request.reply);
                                }
                            },
                            None => break,
                        }
                    },
                    event = swarm.select_next_some() => {
                        handle_event(&mut swarm, &mut pending_requests, event)
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

        self.tx.send(request).await.map_err(anyhow::Error::new)?;

        match reply_rx.await.map_err(anyhow::Error::new)? {
            SwarmResponse::PutRecord(result) => result,
            SwarmResponse::GetRecord(_) => Err(anyhow::anyhow!("unexpected get record response")),
        }
    }

    pub async fn get_value(&self, key: String) -> anyhow::Result<Option<String>> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let request = SwarmRequest {
            command: SwarmCommand::GetRecord { key },
            reply: reply_tx,
        };

        self.tx.send(request).await.map_err(anyhow::Error::new)?;

        match reply_rx.await.map_err(anyhow::Error::new)? {
            SwarmResponse::GetRecord(result) => result,
            SwarmResponse::PutRecord(_) => Err(anyhow::anyhow!("unexpected put record response")),
        }
    }
}

fn handle_event(
    swarm: &mut Swarm<Behaviour>,
    pending_requests: &mut HashMap<kad::QueryId, oneshot::Sender<SwarmResponse>>,
    event: SwarmEvent<BehaviourEvent>,
) {
    match event {
        SwarmEvent::NewListenAddr { .. } => {}
        SwarmEvent::Behaviour(BehaviourEvent::Mdns(event)) => match event {
            mdns::Event::Discovered(list) => {
                for (peer_id, addr) in list {
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr.clone());
                    if let Err(err) = swarm.dial(addr) {
                        tracing::warn!("dial to {peer_id} failed: {err}");
                    }
                }
            }
            mdns::Event::Expired(list) => {
                for (peer_id, addr) in list {
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .remove_address(&peer_id, &addr);
                }
            }
        },
        SwarmEvent::Behaviour(BehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {
            id,
            result,
            ..
        })) => match result {
            kad::QueryResult::PutRecord(outcome) => {
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

                if let Some(reply) = pending_requests.remove(&id) {
                    let _ = reply.send(SwarmResponse::PutRecord(response));
                }
            }
            kad::QueryResult::GetRecord(outcome) => {
                let response = match outcome {
                    Ok(kad::GetRecordOk::FoundRecord(peer_record)) => {
                        match String::from_utf8(peer_record.record.value.clone()) {
                            Ok(value) => {
                                println!(
                                    "Found record {:?}",
                                    std::str::from_utf8(peer_record.record.key.as_ref())
                                        .unwrap_or("<invalid>")
                                );
                                Ok(Some(value))
                            }
                            Err(err) => {
                                eprintln!("Failed to decode record value: {err:?}");
                                Err(err.into())
                            }
                        }
                    }
                    Ok(kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. }) => Ok(None),
                    Err(err) => {
                        eprintln!("Failed to get record: {err:?}");
                        Err(err.into())
                    }
                };

                if let Some(reply) = pending_requests.remove(&id) {
                    let _ = reply.send(SwarmResponse::GetRecord(response));
                }
            }
            _ => {}
        },
        _ => {}
    }
}
