// Copyright 2018 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    error::Error,
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use anyhow::{Result, anyhow};
use axum::{Form, extract::State, http::StatusCode, response::Html, routing::get};
use futures::stream::StreamExt;
use libp2p::{
    Swarm,
    gossipsub::{self},
    mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    select,
    sync::{Mutex, mpsc},
};
use tracing_subscriber::EnvFilter;

// Commands that can be sent to the swarm
#[derive(Debug)]
enum SwarmCommand {
    PublishMessage(String),
    RequestHistory,
    GetPeers(tokio::sync::oneshot::Sender<Vec<String>>),
}

// Message envelope for gossipsub - includes metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
enum GossipEnvelope {
    ChatMessage { content: String, timestamp: u64 },
    HistoryRequest { request_id: u64 }, // Added request_id to avoid duplicates
    HistoryResponse { messages: Vec<GossipMessage> },
}

// Shared application state
#[derive(Clone)]
struct AppState {
    swarm_tx: mpsc::UnboundedSender<SwarmCommand>,
    messages: Arc<Mutex<Vec<GossipMessage>>>,
    peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GossipMessage {
    from_peer: String,
    message: String,
    timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PeerInfo {
    peer_id: String,
    discovered_at: u64,
}

#[derive(Serialize, Deserialize)]
struct PublishRequest {
    message: String,
}

// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

struct MySwarm {
    swarm: Swarm<MyBehaviour>,
    topic: gossipsub::IdentTopic,
    app_state: AppState,
    history_requested: bool, // Track if we've already requested history
}

impl MySwarm {
    fn new(port: u16, app_state: AppState) -> Result<Self> {
        let mut swarm = libp2p::SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_quic()
            .with_behaviour(|key| {
                // To content-address message, we can take the hash of message and use it as an ID.
                let message_id_fn = |message: &gossipsub::Message| {
                    let mut s = DefaultHasher::new();
                    message.data.hash(&mut s);
                    gossipsub::MessageId::from(s.finish().to_string())
                };

                // Set a custom gossipsub configuration
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
                    .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message
                    // signing)
                    .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
                    .build()
                    .map_err(|_| anyhow!("gossipsub"))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

                // build a gossipsub network behaviour
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;
                Ok(MyBehaviour { gossipsub, mdns })
            })?
            .build();

        let topic = gossipsub::IdentTopic::new("pki-topic");
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        swarm.listen_on(format!("/ip4/0.0.0.0/udp/{}/quic-v1", port).parse()?)?;

        Ok(Self {
            swarm,
            topic,
            app_state,
            history_requested: false,
        })
    }

    pub async fn main_loop(
        &mut self,
        mut shutdown_rx: tokio::sync::mpsc::Receiver<()>,
        mut command_rx: mpsc::UnboundedReceiver<SwarmCommand>,
    ) {
        loop {
            select! {
                Some(command) = command_rx.recv() => {
                    self.handle_command(command).await;
                }
                event = self.swarm.select_next_some() => {
                    self.handle_event(event).await;
                }
                _ = shutdown_rx.recv() => {
                    println!("Received shutdown signal, stopping swarm...");
                    break;
                }
            }
        }
    }

    async fn handle_command(&mut self, command: SwarmCommand) {
        match command {
            SwarmCommand::PublishMessage(msg) => {
                let envelope = GossipEnvelope::ChatMessage {
                    content: msg,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                if let Ok(data) = serde_json::to_vec(&envelope) {
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.topic.clone(), data)
                    {
                        eprintln!("Failed to publish message: {e:?}");
                    }
                } else {
                    eprintln!("Failed to serialize message");
                }
            }
            SwarmCommand::RequestHistory => {
                let envelope = GossipEnvelope::HistoryRequest {
                    request_id: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64,
                };
                if let Ok(data) = serde_json::to_vec(&envelope) {
                    if let Err(e) = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.topic.clone(), data)
                    {
                        eprintln!("Failed to publish history request: {e:?}");
                    }
                }
            }
            SwarmCommand::GetPeers(tx) => {
                let peers: Vec<String> =
                    self.app_state.peers.lock().await.keys().cloned().collect();
                let _ = tx.send(peers);
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<MyBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mDNS discovered a new peer: {peer_id}");
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);

                    // Add to shared state
                    let peer_info = PeerInfo {
                        peer_id: peer_id.to_string(),
                        discovered_at: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };
                    self.app_state
                        .peers
                        .lock()
                        .await
                        .insert(peer_id.to_string(), peer_info);
                }
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed {
                peer_id,
                topic,
            })) => {
                println!("🔔 Peer {peer_id} subscribed to topic: {topic}");

                // Only request history once, when we're a new joiner with no messages
                let message_count = self.app_state.messages.lock().await.len();

                println!(
                    "   Current state: {} messages, history_requested: {}",
                    message_count, self.history_requested
                );

                if !self.history_requested && message_count == 0 {
                    self.history_requested = true;
                    println!("📨 Requesting message history from network...");
                    let request = GossipEnvelope::HistoryRequest {
                        request_id: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64,
                    };
                    if let Ok(data) = serde_json::to_vec(&request) {
                        match self
                            .swarm
                            .behaviour_mut()
                            .gossipsub
                            .publish(self.topic.clone(), data)
                        {
                            Ok(_) => println!("✅ History request sent successfully"),
                            Err(e) => println!("❌ Failed to send history request: {e:?}"),
                        }
                    }
                }
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                for (peer_id, _multiaddr) in list {
                    println!("mDNS discover peer has expired: {peer_id}");
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);

                    // Remove from shared state
                    self.app_state
                        .peers
                        .lock()
                        .await
                        .remove(&peer_id.to_string());
                }
            }
            SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id: id,
                message,
            })) => {
                // Deserialize the envelope
                match serde_json::from_slice::<GossipEnvelope>(&message.data) {
                    Ok(envelope) => {
                        match envelope {
                            GossipEnvelope::ChatMessage { content, timestamp } => {
                                println!(
                                    "Got message: '{}' with id: {id} from peer: {peer_id}",
                                    content
                                );

                                // Store in shared state
                                let gossip_msg = GossipMessage {
                                    from_peer: peer_id.to_string(),
                                    message: content,
                                    timestamp,
                                };
                                self.app_state.messages.lock().await.push(gossip_msg);
                            }
                            GossipEnvelope::HistoryRequest { request_id } => {
                                println!(
                                    "📥 Received history request (id: {}) from peer: {peer_id}",
                                    request_id
                                );

                                // Get current message history
                                let messages = self.app_state.messages.lock().await.clone();
                                println!(
                                    "📤 Sending history response with {} messages",
                                    messages.len()
                                );

                                // Send history response
                                let response = GossipEnvelope::HistoryResponse {
                                    messages: messages.clone(),
                                };
                                if let Ok(data) = serde_json::to_vec(&response) {
                                    match self
                                        .swarm
                                        .behaviour_mut()
                                        .gossipsub
                                        .publish(self.topic.clone(), data)
                                    {
                                        Ok(_) => println!("✅ History response sent successfully"),
                                        Err(e) => {
                                            println!("❌ Failed to send history response: {e:?}")
                                        }
                                    }
                                } else {
                                    println!("❌ Failed to serialize history response");
                                }
                            }
                            GossipEnvelope::HistoryResponse { messages } => {
                                println!(
                                    "📥 Received history response with {} messages from peer: {peer_id}",
                                    messages.len()
                                );

                                // Merge messages into local state (avoiding duplicates)
                                let mut local_messages = self.app_state.messages.lock().await;
                                let initial_count = local_messages.len();

                                for msg in messages {
                                    // Simple deduplication: check if message already exists
                                    let exists = local_messages.iter().any(|m| {
                                        m.from_peer == msg.from_peer
                                            && m.message == msg.message
                                            && m.timestamp == msg.timestamp
                                    });
                                    if !exists {
                                        local_messages.push(msg);
                                    }
                                }
                                // Sort by timestamp
                                local_messages.sort_by_key(|m| m.timestamp);

                                let added_count = local_messages.len() - initial_count;
                                println!(
                                    "✅ Added {} new messages to history (total: {})",
                                    added_count,
                                    local_messages.len()
                                );
                            }
                        }
                    }
                    Err(e) => {
                        // Fallback for old-style messages (raw strings)
                        let msg_str = String::from_utf8_lossy(&message.data).to_string();
                        println!(
                            "Got legacy message: '{}' with id: {id} from peer: {peer_id} (parse error: {e})",
                            msg_str
                        );

                        // Store as legacy message
                        let gossip_msg = GossipMessage {
                            from_peer: peer_id.to_string(),
                            message: msg_str,
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };
                        self.app_state.messages.lock().await.push(gossip_msg);
                    }
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Local node is listening on {address}");
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let listener = TcpListener::bind("0.0.0.0:0").await?;
    let port = listener.local_addr()?.port();

    // Create shared state and channels
    let (swarm_tx, swarm_rx) = mpsc::unbounded_channel::<SwarmCommand>();
    let app_state = AppState {
        swarm_tx,
        messages: Arc::new(Mutex::new(Vec::new())),
        peers: Arc::new(Mutex::new(HashMap::new())),
    };

    let mut swarm = MySwarm::new(port, app_state.clone())?;

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Spawn the swarm event loop in a separate task
    let swarm_handle = tokio::spawn(async move {
        swarm.main_loop(shutdown_rx, swarm_rx).await;
    });

    // Build router with state
    let router = axum::Router::new()
        .route("/", get(index))
        .route("/messages", get(get_messages))
        .route("/peers", get(get_peers))
        .route("/publish", get(publish_form).post(publish_message))
        .with_state(app_state);

    println!("HTTP server running on http://localhost:{}", port);

    // Spawn axum server
    let _server_handle = tokio::spawn(async move { axum::serve(listener, router).await });

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    // Send shutdown signal
    let _ = shutdown_tx.send(()).await;

    // Wait for swarm task to finish
    let _ = swarm_handle.await;

    // Note: server_handle will be dropped/cancelled here

    Ok(())
}

// HTTP handlers
async fn index() -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Gossipsub Node</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #333; }}
        .links {{ list-style: none; padding: 0; }}
        .links li {{ margin: 10px 0; }}
        .links a {{ 
            display: inline-block;
            padding: 10px 20px;
            background: #007bff;
            color: white;
            text-decoration: none;
            border-radius: 5px;
        }}
        .links a:hover {{ background: #0056b3; }}
    </style>
</head>
<body>
    <h1>🌐 Gossipsub Node Dashboard</h1>
    <p>Welcome to your libp2p gossipsub node interface!</p>
    <ul class="links">
        <li><a href="/messages">📨 View Messages</a></li>
        <li><a href="/peers">👥 View Peers</a></li>
        <li><a href="/publish">✉️ Publish Message</a></li>
    </ul>
</body>
</html>"#
    ))
}

async fn get_messages(State(state): State<AppState>) -> Html<String> {
    let messages = state.messages.lock().await.clone();

    let messages_html = if messages.is_empty() {
        "<p>No messages received yet.</p>".to_string()
    } else {
        messages
            .iter()
            .map(|msg| {
                format!(
                    r#"<div class="message">
                        <div class="from">From: {}</div>
                        <div class="content">{}</div>
                        <div class="time">Time: {}</div>
                    </div>"#,
                    msg.from_peer, msg.message, msg.timestamp
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Messages - Gossipsub Node</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #333; }}
        .back {{ margin: 20px 0; }}
        .back a {{ color: #007bff; text-decoration: none; }}
        .message {{ 
            border: 1px solid #ddd;
            padding: 15px;
            margin: 10px 0;
            border-radius: 5px;
            background: #f9f9f9;
        }}
        .from {{ font-weight: bold; color: #555; }}
        .content {{ margin: 10px 0; font-size: 16px; }}
        .time {{ font-size: 12px; color: #999; }}
    </style>
</head>
<body>
    <h1>📨 Received Messages</h1>
    <div class="back"><a href="/">← Back to Dashboard</a></div>
    {}
</body>
</html>"#,
        messages_html
    ))
}

async fn get_peers(State(state): State<AppState>) -> Html<String> {
    let peers: Vec<PeerInfo> = state.peers.lock().await.values().cloned().collect();

    let peers_html = if peers.is_empty() {
        "<p>No peers discovered yet.</p>".to_string()
    } else {
        peers
            .iter()
            .map(|peer| {
                format!(
                    r#"<div class="peer">
                        <div class="peer-id">{}</div>
                        <div class="discovered">Discovered: {}</div>
                    </div>"#,
                    peer.peer_id, peer.discovered_at
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Peers - Gossipsub Node</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #333; }}
        .back {{ margin: 20px 0; }}
        .back a {{ color: #007bff; text-decoration: none; }}
        .peer {{ 
            border: 1px solid #ddd;
            padding: 15px;
            margin: 10px 0;
            border-radius: 5px;
            background: #f0f8ff;
        }}
        .peer-id {{ 
            font-family: monospace;
            font-size: 14px;
            word-break: break-all;
            color: #333;
        }}
        .discovered {{ 
            font-size: 12px;
            color: #999;
            margin-top: 5px;
        }}
    </style>
</head>
<body>
    <h1>👥 Discovered Peers</h1>
    <div class="back"><a href="/">← Back to Dashboard</a></div>
    <p>Total peers: {}</p>
    {}
</body>
</html>"#,
        peers.len(),
        peers_html
    ))
}

async fn publish_message(
    State(state): State<AppState>,
    Form(req): Form<PublishRequest>,
) -> Result<Html<String>, StatusCode> {
    // Send publish command to swarm
    state
        .swarm_tx
        .send(SwarmCommand::PublishMessage(req.message.clone()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Message Sent - Gossipsub Node</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #333; }}
        .success {{ 
            padding: 20px;
            background: #d4edda;
            border: 1px solid #c3e6cb;
            border-radius: 5px;
            color: #155724;
        }}
        .back {{ margin: 20px 0; }}
        .back a {{ color: #007bff; text-decoration: none; }}
    </style>
</head>
<body>
    <h1>✅ Message Sent!</h1>
    <div class="success">
        <p>Your message has been published to the gossipsub network:</p>
        <p><strong>{}</strong></p>
    </div>
    <div class="back">
        <a href="/publish">← Send Another Message</a> | 
        <a href="/">Dashboard</a>
    </div>
</body>
</html>"#,
        req.message
    )))
}

async fn publish_form() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Publish Message - Gossipsub Node</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #333; }}
        .back {{ margin: 20px 0; }}
        .back a {{ color: #007bff; text-decoration: none; }}
        form {{ 
            border: 1px solid #ddd;
            padding: 20px;
            border-radius: 5px;
            background: #f9f9f9;
        }}
        label {{ 
            display: block;
            margin-bottom: 10px;
            font-weight: bold;
        }}
        textarea {{ 
            width: 100%;
            min-height: 100px;
            padding: 10px;
            border: 1px solid #ccc;
            border-radius: 4px;
            font-family: Arial, sans-serif;
            font-size: 14px;
            box-sizing: border-box;
        }}
        button {{ 
            margin-top: 10px;
            padding: 10px 20px;
            background: #28a745;
            color: white;
            border: none;
            border-radius: 5px;
            cursor: pointer;
            font-size: 16px;
        }}
        button:hover {{ background: #218838; }}
    </style>
</head>
<body>
    <h1>✉️ Publish Message</h1>
    <div class="back"><a href="/">← Back to Dashboard</a></div>
    <form method="POST" action="/publish">
        <label for="message">Message:</label>
        <textarea id="message" name="message" placeholder="Type your message here..." required></textarea>
        <button type="submit">📤 Send Message</button>
    </form>
</body>
</html>"#,
    )
}
