use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::sync::Arc;

use anyhow::Result;
use diesel::{PgConnection, r2d2::ConnectionManager};
use ed25519_dalek::SigningKey;
use diesel::{ExpressionMethods, RunQueryDsl};
use end2::{
    App, AppState, AuthService, ChainDeviceKeyService, DbAuthService, DbMessageRelayService,
    DbOtkService, InboundUser, OAuthHandler,
};
use miner::http::MinerApi;
use miner::network::Node;
use miner::{Chain, GenesisDevice, create_genesis};
use rand_core::OsRng;
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use uuid::Uuid;
use vodozemac::{Curve25519PublicKey, olm::Session};

use device::{DecryptedMessage, Device, DeviceInfo, InboundChatMessage, MessagePayload, Otk};

/// Backend attestation key used across chain-backed test environments.
pub struct BackendKeys {
    pub signing_key: SigningKey,
    pub verifying_key: ed25519_dalek::VerifyingKey,
}

pub mod device;

/// Bind to port 0 and return the listener + the OS-assigned port.
async fn random_listener() -> (TcpListener, u16) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("TcpListener bind to port 0");
    let port = listener.local_addr().expect("local_addr").port();
    (listener, port)
}

/// Spawns the app on an OS-assigned port and returns the port number.
pub async fn spawn_app() -> u16 {
    let (listener, port) = random_listener().await;

    let oauth = OAuthHandler::new(String::new(), String::new(), String::new());

    let database_url = "postgres://postgres@localhost/postgres".to_string();
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to connect to Postgres");

    let app = App::new(oauth, pool);
    tokio::spawn(async move { app.run(listener).await });

    port
}

pub struct ChainTestUser {
    pub username: String,
    pub password: String,
    pub user_id: Uuid,
    pub device: Device,
    pub chain_signing_key: SigningKey,
}

/// Result of spawning the chain-backed test environment.
pub struct ChainTestEnv {
    pub backend_port: u16,
    pub miner_port: u16,
    pub test_users: Vec<ChainTestUser>,
    pub backend_keys: BackendKeys,
}

/// Spawns a miner HTTP API + backend with `ChainDeviceKeyService`.
///
/// Genesis contains only a bootstrap authority. Users must register their
/// devices on-chain via the miner HTTP API (POST /tx + POST /mine).
pub async fn spawn_app_with_chain() -> ChainTestEnv {
    let (backend_listener, backend_port) = random_listener().await;
    let (miner_listener, miner_port) = random_listener().await;

    let oauth = OAuthHandler::new(String::new(), String::new(), String::new());

    let database_url = "postgres://postgres@localhost/postgres".to_string();
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to connect to Postgres");

    // Phase 1: Register users in DB to get their user_ids
    let auth = Arc::new(DbAuthService::new(pool.clone()));

    let id = Uuid::now_v7().simple().to_string();
    let suffix = &id[..16];
    let users_to_create = vec![
        (format!("a{suffix}1"), "pass"),
        (format!("a{suffix}2"), "pass"),
    ];

    let mut test_users = Vec::new();

    for (username, password) in &users_to_create {
        let user = auth
            .register_user(InboundUser {
                username: username.clone(),
                password: password.to_string(),
                confirm_password: password.to_string(),
            })
            .await
            .expect("Failed to register test user");

        let device_id = Uuid::now_v7();
        let device = Device::new(device_id);
        let chain_signing_key = SigningKey::generate(&mut OsRng);

        test_users.push(ChainTestUser {
            username: username.clone(),
            password: password.to_string(),
            user_id: user.id,
            device,
            chain_signing_key,
        });
    }

    // Phase 2: Insert device records into DB (needed for OTK foreign keys + signature verification)
    {
        let mut conn = pool.get().expect("Failed to get DB connection");
        for tu in &test_users {
            diesel::insert_into(end2::device::table)
                .values((
                    end2::device::id.eq(tu.device.id()),
                    end2::device::user_id.eq(tu.user_id),
                    end2::device::ed25519.eq(Some(tu.device.ed25519_public_key_bytes().to_vec())),
                    end2::device::x25519.eq(Some(tu.device.x25519_public_key_bytes().to_vec())),
                ))
                .execute(&mut conn)
                .expect("Failed to insert device into DB");
        }
    }

    // Phase 3: Generate backend attestation key pair
    let backend_signing_key = SigningKey::generate(&mut OsRng);
    let backend_verifying_key = backend_signing_key.verifying_key();

    // Phase 4: Create genesis with a bootstrap authority only (no user devices).
    let bootstrap_key = SigningKey::generate(&mut OsRng);
    let bootstrap_device = GenesisDevice {
        user_id: Uuid::now_v7(),
        device_id: Uuid::now_v7(),
        signing_key: bootstrap_key.clone(),
        x25519: [0u8; 32], // placeholder — bootstrap device is just for block signing
    };

    let genesis = create_genesis(&bootstrap_key, 1, &[bootstrap_device], None)
        .expect("Failed to create genesis block");
    let chain = Chain::new(genesis, Some(backend_verifying_key))
        .expect("Failed to create chain");
    let chain = Arc::new(RwLock::new(chain));

    // Phase 5: Start miner HTTP API (shares chain)
    let miner_api = MinerApi::new(Arc::clone(&chain), bootstrap_key, Some(backend_verifying_key));
    tokio::spawn(async move { miner_api.run(miner_listener).await });

    // Phase 6: Start backend with ChainDeviceKeyService (shares same chain)
    let device_keys = Arc::new(ChainDeviceKeyService::new(Arc::clone(&chain)));
    let otks = Arc::new(DbOtkService::new(pool.clone()));
    let relay = Arc::new(DbMessageRelayService::new(pool.clone()));
    let mut app_state = AppState::new(auth, device_keys, otks, relay, oauth, pool);
    app_state.attestation_key = Some(Arc::new(backend_signing_key.clone()));

    let app = App::from_state(app_state);
    tokio::spawn(async move { app.run(backend_listener).await });

    let backend_keys = BackendKeys {
        signing_key: backend_signing_key,
        verifying_key: backend_verifying_key,
    };

    ChainTestEnv {
        backend_port,
        miner_port,
        test_users,
        backend_keys,
    }
}

/// Result of spawning a multi-miner P2P test environment.
pub struct MultiMinerEnv {
    pub backend_port: u16,
    pub miner_ports: Vec<u16>,
    pub test_users: Vec<ChainTestUser>,
    pub backend_keys: BackendKeys,
}

/// Spawns `n` P2P miner nodes + 1 backend.
///
/// Each miner gets its own `Chain` (from the same genesis), `Node`, and HTTP API
/// in integrated mode (txs forwarded to node via channel, no `/mine` endpoint).
/// mDNS handles peer discovery automatically on localhost.
pub async fn spawn_app_with_miners(n: usize) -> MultiMinerEnv {
    let (backend_listener, backend_port) = random_listener().await;

    let oauth = OAuthHandler::new(String::new(), String::new(), String::new());

    let database_url = "postgres://postgres@localhost/postgres".to_string();
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to connect to Postgres");

    // Phase 1: Generate backend attestation key pair
    let backend_signing_key = SigningKey::generate(&mut OsRng);
    let backend_verifying_key = backend_signing_key.verifying_key();

    // Phase 2: Create n miner signing keys as genesis devices (each becomes an authority)
    let mut miner_keys = Vec::with_capacity(n);
    let mut genesis_devices = Vec::with_capacity(n);
    for _ in 0..n {
        let key = SigningKey::generate(&mut OsRng);
        genesis_devices.push(GenesisDevice {
            user_id: Uuid::now_v7(),
            device_id: Uuid::now_v7(),
            signing_key: key.clone(),
            x25519: [0u8; 32],
        });
        miner_keys.push(key);
    }

    let genesis = create_genesis(
        &miner_keys[0],
        1,
        &genesis_devices,
        Some(&backend_signing_key),
    )
    .expect("Failed to create genesis block");

    // Phase 3: Create each miner node, resolve listen addresses, and connect them
    let mut miner_ports = Vec::with_capacity(n);
    let mut first_chain = None;
    let mut nodes = Vec::with_capacity(n);
    let mut node_addrs = Vec::with_capacity(n);

    for (i, key) in miner_keys.into_iter().enumerate() {
        let chain = Chain::new(genesis.clone(), Some(backend_verifying_key))
            .expect("Failed to create chain");
        let chain = Arc::new(RwLock::new(chain));

        if i == 0 {
            first_chain = Some(Arc::clone(&chain));
        }

        let (mut node, tx_sender) =
            Node::new(Arc::clone(&chain), key).expect("Failed to create node");
        node.listen("/ip4/127.0.0.1/tcp/0")
            .expect("Failed to listen");

        // Wait for the swarm to resolve the actual listen address
        let addr = node.wait_for_listen_addr().await;
        let peer_id = node.peer_id();

        let miner_api =
            MinerApi::integrated(Arc::clone(&chain), tx_sender, Some(backend_verifying_key));
        let (http_listener, http_port) = random_listener().await;
        miner_ports.push(http_port);

        node_addrs.push((peer_id, addr));
        nodes.push((node, miner_api, http_listener));
    }

    // Manually connect all nodes to each other (full mesh)
    for i in 0..nodes.len() {
        for j in 0..node_addrs.len() {
            if i != j {
                let (_, ref addr) = node_addrs[j];
                nodes[i]
                    .0
                    .dial(addr.clone())
                    .expect("Failed to dial peer");
                nodes[i].0.add_explicit_peer(&node_addrs[j].0);
            }
        }
    }

    // Spawn all nodes
    for (mut node, miner_api, http_listener) in nodes {
        tokio::spawn(async move { node.run(Duration::from_millis(500)).await });
        tokio::spawn(async move { miner_api.run(http_listener).await });
    }

    // Phase 4: Register test users in DB
    let auth = Arc::new(DbAuthService::new(pool.clone()));

    let id = Uuid::now_v7().simple().to_string();
    let suffix = &id[..16];
    let users_to_create = vec![
        (format!("a{suffix}1"), "pass"),
        (format!("a{suffix}2"), "pass"),
    ];

    let mut test_users = Vec::new();

    for (username, password) in &users_to_create {
        let user = auth
            .register_user(InboundUser {
                username: username.clone(),
                password: password.to_string(),
                confirm_password: password.to_string(),
            })
            .await
            .expect("Failed to register test user");

        let device_id = Uuid::now_v7();
        let device = Device::new(device_id);
        let chain_signing_key = SigningKey::generate(&mut OsRng);

        test_users.push(ChainTestUser {
            username: username.clone(),
            password: password.to_string(),
            user_id: user.id,
            device,
            chain_signing_key,
        });
    }

    // Insert device records into DB
    {
        let mut conn = pool.get().expect("Failed to get DB connection");
        for tu in &test_users {
            diesel::insert_into(end2::device::table)
                .values((
                    end2::device::id.eq(tu.device.id()),
                    end2::device::user_id.eq(tu.user_id),
                    end2::device::ed25519.eq(Some(tu.device.ed25519_public_key_bytes().to_vec())),
                    end2::device::x25519.eq(Some(tu.device.x25519_public_key_bytes().to_vec())),
                ))
                .execute(&mut conn)
                .expect("Failed to insert device into DB");
        }
    }

    // Phase 5: Start backend with ChainDeviceKeyService pointing to miner 0's chain
    let chain_for_backend = first_chain.expect("at least one miner");
    let device_keys = Arc::new(ChainDeviceKeyService::new(chain_for_backend));
    let otks = Arc::new(DbOtkService::new(pool.clone()));
    let relay = Arc::new(DbMessageRelayService::new(pool.clone()));
    let mut app_state = AppState::new(auth, device_keys, otks, relay, oauth, pool);
    app_state.attestation_key = Some(Arc::new(backend_signing_key.clone()));

    let app = App::from_state(app_state);
    tokio::spawn(async move { app.run(backend_listener).await });

    // Phase 6: Wait for gossipsub mesh to establish (nodes are manually connected)
    tokio::time::sleep(Duration::from_secs(3)).await;

    let backend_keys = BackendKeys {
        signing_key: backend_signing_key,
        verifying_key: backend_verifying_key,
    };

    MultiMinerEnv {
        backend_port,
        miner_ports,
        test_users,
        backend_keys,
    }
}

#[derive(Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
}

#[derive(Deserialize)]
pub struct ChannelInfo {
    pub channel_id: Uuid,
    pub participants: Vec<UserInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelId {
    pub channel_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub message_id: Uuid,
    pub device_id: Uuid,
    pub channel_id: Uuid,
    pub payloads: Vec<MessagePayload>,
}

pub struct ApiClient {
    client: Client,
    device: Device,
    // device_id, session
    sessions: HashMap<Uuid, Session>,
    // channel_id, (message_id, message)
    histories: HashMap<Uuid, BTreeMap<Uuid, DecryptedMessage>>,
    user_id: Uuid,
    username: String,
    password: String,
    base_url: String,
}

impl ApiClient {
    pub async fn new(username: &str, password: &str, device: Option<Device>) -> Result<Self> {
        Self::with_port(username, password, device, 8081).await
    }

    /// Create a client for a user that was already registered directly (e.g. via DbAuthService).
    /// Skips register/login HTTP calls — uses basic auth for all subsequent requests.
    pub fn preconfigured(
        username: &str,
        password: &str,
        user_id: Uuid,
        device: Device,
        port: u16,
    ) -> Self {
        Self {
            client: Client::new(),
            device,
            sessions: HashMap::new(),
            histories: HashMap::new(),
            user_id,
            username: username.to_string(),
            password: password.to_string(),
            base_url: format!("http://localhost:{port}/api"),
        }
    }

    pub async fn with_port(
        username: &str,
        password: &str,
        device: Option<Device>,
        port: u16,
    ) -> Result<Self> {
        let base_url = format!("http://localhost:{port}/api");
        let client = Client::new();
        let response = client
            .post(format!("{base_url}/auth/register"))
            .json(&serde_json::json!({
                "username": username,
                "password": password,
                "confirm_password": password,
            }))
            .send()
            .await?;

        let response = if !response.status().is_success() {
            client
                .post(format!("{base_url}/auth/login"))
                .json(&serde_json::json!({
                    "username": username,
                    "password": password,
                }))
                .send()
                .await?
        } else {
            response
        };

        let user_info = response.error_for_status()?.json::<UserInfo>().await?;

        let device = if let Some(device) = device {
            device
        } else {
            let device_info = client
                .post(format!("{base_url}/me/device"))
                .basic_auth(username, Some(password))
                .send()
                .await?
                .error_for_status()?
                .json::<DeviceInfo>()
                .await?;

            let device = Device::new(device_info.device_id);

            let _ = client
                .put(format!("{base_url}/me/device/{}", device_info.device_id))
                .basic_auth(username, Some(password))
                .json(&device.get_identity_keys())
                .send()
                .await?
                .error_for_status()?;

            device
        };

        Ok(Self {
            client,
            device,
            sessions: HashMap::new(),
            histories: HashMap::new(),
            user_id: user_info.id,
            username: username.to_string(),
            password: password.to_string(),
            base_url,
        })
    }

    fn get(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .get(format!("{}{endpoint}", self.base_url))
            .basic_auth(&self.username, Some(&self.password))
    }

    fn put(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .put(format!("{}{endpoint}", self.base_url))
            .basic_auth(&self.username, Some(&self.password))
    }

    fn post(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .post(format!("{}{endpoint}", self.base_url))
            .basic_auth(&self.username, Some(&self.password))
    }

    /// Send a raw POST request and return the response (for testing error paths).
    pub async fn raw_post(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response> {
        Ok(self.post(endpoint).json(body).send().await?)
    }

    /// Send a raw GET request and return the response (for testing error paths).
    pub async fn raw_get(&self, endpoint: &str) -> Result<reqwest::Response> {
        Ok(self.get(endpoint).send().await?)
    }

    pub async fn upload_otks(&mut self) -> Result<()> {
        self.post(&format!("/me/device/{}/otks", self.device.id()))
            .json(&self.device.get_otks(10))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn create_channel(&self, recipient: &str) -> Result<()> {
        self.post("/channel")
            .json(&serde_json::json!({
                "recipient": recipient
            }))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn channels(&self) -> Result<Vec<ChannelId>> {
        let response = self
            .get("/me/channels")
            .send()
            .await?
            .json::<Vec<ChannelId>>()
            .await?;

        Ok(response)
    }

    pub async fn get_channel_participants(&self, channel: &ChannelId) -> Result<ChannelInfo> {
        let response = self
            .get(&format!("/channel/{}", channel.channel_id))
            .send()
            .await?
            .json::<ChannelInfo>()
            .await?;

        Ok(response)
    }

    pub async fn get_user_info(&self, user_id: Uuid) -> Result<UserInfo> {
        let response = self
            .get(&format!("/user/{}", user_id))
            .send()
            .await?
            .json::<UserInfo>()
            .await?;

        Ok(response)
    }

    pub async fn get_user_devices(&self, user_id: Uuid) -> Result<Vec<DeviceInfo>> {
        let response = self
            .get(&format!("/user/{}/devices", user_id))
            .send()
            .await?
            .json::<Vec<DeviceInfo>>()
            .await?;

        Ok(response)
    }

    pub async fn get_device_info(&self, user_id: Uuid, device_id: Uuid) -> Result<DeviceInfo> {
        let response = self
            .get(&format!("/user/{}/device/{}", user_id, device_id))
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            let text = response.text().await?;
            panic!("{}", text);
        }

        match response.json::<DeviceInfo>().await {
            Ok(info) => return Ok(info),
            Err(e) => panic!("{:?}", e),
        }
    }

    pub async fn get_device_otk(&self, user_id: Uuid, device_id: Uuid) -> Result<Otk> {
        let response = self
            .post(&format!("/user/{}/device/{}/otk", user_id, device_id))
            .send()
            .await?
            .json::<Otk>()
            .await?;

        Ok(response)
    }

    pub async fn send_message(
        &mut self,
        channel_info: &ChannelInfo,
        plaintext: &str,
    ) -> Result<()> {
        let mut payloads = vec![];

        for participant in &channel_info.participants {
            let device_ids = self.get_user_devices(participant.id).await?;

            for device in device_ids {
                // Skip our own device — we can't decrypt our own messages
                // (the inbound/outbound session pair gets overwritten)
                if device.device_id == self.device.id() {
                    continue;
                }

                let (session, payload) =
                    if let Some(session) = self.sessions.remove(&device.device_id) {
                        if session.has_received_message() {
                            self.device.encrypt(session, &device, plaintext)
                        } else {
                            panic!("Must wait for other user to reply")
                        }
                    } else {
                        let otk = self
                            .get_device_otk(participant.id, device.device_id)
                            .await?;

                        self.device.encrypt_otk(
                            &device,
                            plaintext,
                            Curve25519PublicKey::from_base64(&otk.otk)?,
                        )
                    }?;

                self.sessions.insert(device.device_id, session);
                payloads.push(payload);
            }
        }

        let message = ChatMessage {
            message_id: Uuid::now_v7(),
            device_id: self.device.id(),
            channel_id: channel_info.channel_id,
            payloads,
        };

        self.post(&format!("/channel/{}/msg", channel_info.channel_id))
            .json(&message)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn get_history(&mut self, channel_id: Uuid) -> Result<Vec<DecryptedMessage>> {
        let last_message_id = self
            .histories
            .get(&channel_id)
            .and_then(|h| h.keys().last().copied());

        let mut url = format!(
            "/channel/{}/history?device={}",
            channel_id,
            self.device.id()
        );

        if let Some(last) = last_message_id {
            let _ = write!(url, "&after={}", last);
        }

        let messages = self
            .get(&url)
            .send()
            .await?
            .json::<Vec<InboundChatMessage>>()
            .await?;

        let mut decrypted_messages = Vec::with_capacity(messages.len());
        let mut devices = HashMap::<Uuid, DeviceInfo>::new();

        for message in messages {
            let device_id = message.device_id;
            let device_info = if let Some(device_info) = devices.get(&device_id) {
                device_info.clone()
            } else {
                let device_info = self.get_device_info(message.author_id, device_id).await?;

                devices.insert(device_id, device_info.clone());
                device_info
            };

            let (session, plaintext) = if message.is_pre_key {
                self.device.decrypt_otk(&device_info, message)?
            } else {
                let session = self
                    .sessions
                    .remove(&message.device_id)
                    .expect("Missing session");
                self.device.decrypt(session, &device_info, message)?
            };

            self.sessions.insert(device_id, session);
            decrypted_messages.push(plaintext);
        }

        let history = self.histories.entry(channel_id).or_default();

        for plaintext in decrypted_messages {
            history.insert(plaintext.message_id, plaintext);
        }

        Ok(history.values().cloned().collect())
    }

    pub async fn me(&self) -> Result<UserInfo> {
        Ok(self
            .get("/api/me")
            .send()
            .await?
            .error_for_status()?
            .json::<UserInfo>()
            .await?)
    }
}
