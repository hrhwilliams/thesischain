use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use anyhow::Result;
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use vodozemac::{Curve25519PublicKey, olm::Session};

use crate::device::{
    DecryptedMessage, Device, DeviceInfo, InboundChatMessage, MessagePayload, Otk,
};

mod device;

#[derive(Deserialize)]
struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
}

#[derive(Deserialize)]
struct ChannelInfo {
    pub channel_id: Uuid,
    pub participants: Vec<UserInfo>,
}

#[derive(Debug, Deserialize)]
struct ChannelId {
    pub channel_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub message_id: Uuid,
    pub device_id: Uuid,
    pub channel_id: Uuid,
    pub payloads: Vec<MessagePayload>,
}

struct ApiClient {
    client: Client,
    device: Device,
    // device_id, session
    sessions: HashMap<Uuid, Session>,
    // channel_id, (message_id, message)
    histories: HashMap<Uuid, BTreeMap<Uuid, DecryptedMessage>>,
    user_id: Uuid,
    username: String,
    password: String,
}

impl ApiClient {
    async fn new(username: &str, password: &str, device: Option<Device>) -> Result<Self> {
        let client = Client::new();
        let response = client
            .post("http://localhost:8081/api/auth/register")
            .json(&serde_json::json!({
                "username": username,
                "password": password,
                "confirm_password": password,
            }))
            .send()
            .await?;

        let response = if response.status() == 500 {
            client
                .post("http://localhost:8081/api/auth/login")
                .json(&serde_json::json!({
                    "username": username,
                    "password": password,
                }))
                .send()
                .await?
        } else {
            response
        };

        let user_info = response
            .error_for_status()?
            .json::<UserInfo>()
            .await?;

        let device = if device.is_some() {
            // SAFETY: checked in if statement above
            unsafe { device.unwrap_unchecked() }
        } else {
            let device_info = client
                .post("http://localhost:8081/api/me/device")
                .basic_auth(username, Some(password))
                .send()
                .await?
                .error_for_status()?
                .json::<DeviceInfo>()
                .await?;

            let device = Device::new(device_info.device_id);

            let _ = client
                .put(&format!(
                    "http://localhost:8081/api/me/device/{}",
                    device_info.device_id
                ))
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
        })
    }

    fn get(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .get(format!("http://localhost:8081/api{}", endpoint))
            .basic_auth(&self.username, Some(&self.password))
    }

    fn put(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .put(format!("http://localhost:8081/api{}", endpoint))
            .basic_auth(&self.username, Some(&self.password))
    }

    fn post(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .post(format!("http://localhost:8081/api{}", endpoint))
            .basic_auth(&self.username, Some(&self.password))
    }

    async fn upload_otks(&mut self) -> Result<()> {
        self.post(&format!("/me/device/{}/otks", self.device.id()))
            .json(&self.device.get_otks(10))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn create_channel(&self, recipient: &str) -> Result<()> {
        self.post("/channel")
            .json(&serde_json::json!({
                "recipient": recipient
            }))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn channels(&self) -> Result<Vec<ChannelId>> {
        let response = self
            .get("/me/channels")
            .send()
            .await?
            .json::<Vec<ChannelId>>()
            .await?;

        Ok(response)
    }

    async fn get_channel_participants(&self, channel: &ChannelId) -> Result<ChannelInfo> {
        let response = self
            .get(&format!("/channel/{}", channel.channel_id))
            .send()
            .await?
            .json::<ChannelInfo>()
            .await?;

        Ok(response)
    }

    async fn get_user_info(&self, user_id: Uuid) -> Result<UserInfo> {
        let response = self
            .get(&format!("/user/{}", user_id))
            .send()
            .await?
            .json::<UserInfo>()
            .await?;

        Ok(response)
    }

    async fn get_user_devices(&self, user_id: Uuid) -> Result<Vec<DeviceInfo>> {
        let response = self
            .get(&format!("/user/{}/devices", user_id))
            .send()
            .await?
            .json::<Vec<DeviceInfo>>()
            .await?;

        Ok(response)
    }

    async fn get_device_info(&self, user_id: Uuid, device_id: Uuid) -> Result<DeviceInfo> {
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

    async fn get_device_otk(&self, user_id: Uuid, device_id: Uuid) -> Result<Otk> {
        let response = self
            .post(&format!("/user/{}/device/{}/otk", user_id, device_id))
            .send()
            .await?
            .json::<Otk>()
            .await?;

        Ok(response)
    }

    async fn send_message(&mut self, channel_info: &ChannelInfo, plaintext: &str) -> Result<()> {
        let mut payloads = vec![];

        for participant in &channel_info.participants {
            let device_ids = self.get_user_devices(participant.id).await?;

            for device in device_ids {
                let (session, payload) = if let Some(session) = self.sessions.remove(&device.device_id) {
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

    async fn get_history(&mut self, channel_id: Uuid) -> Result<Vec<DecryptedMessage>> {
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

    async fn me(&self) -> Result<UserInfo> {
        Ok(self
            .get("/api/me")
            .send()
            .await?
            .error_for_status()?
            .json::<UserInfo>()
            .await?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create users
    let mut client1 = ApiClient::new("maki1", "abc", None).await?;
    let mut client2 = ApiClient::new("maki2", "abc", None).await?;

    // Upload one-time keys for users
    client1.upload_otks().await?;
    client2.upload_otks().await?;

    // Client 1 creates a channel to message Client 2 through
    client1.create_channel("maki2").await?;

    // Client 2 lists the channels they are in and selects the first one
    let channels1 = client1.channels().await?;
    let channel1_id = channels1.first().unwrap();
    let channel1_info = client1.get_channel_participants(channel1_id).await?;

    // Client 1 sends a message to the channel
    client1.send_message(&channel1_info, "hello").await?;

    let history = client1.get_history(channel1_id.channel_id).await?;

    println!("Client 1 history ---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }

    // Client 2 lists the channels they are in, selects the first one, and reads the history
    let channels2 = client2.channels().await?;
    let channel2_id = channels2.first().unwrap();
    let channel2_info = client2.get_channel_participants(channel2_id).await?;

    let history = client2.get_history(channel2_id.channel_id).await?;

    println!("Client 2 history ---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }
    println!("---");

    // Client 2 responds
    client2.send_message(&channel1_info, "hello!").await?;

    // Client 1 reads the history
    let history = client1.get_history(channel1_id.channel_id).await?;

    println!("Client 1 history ---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }

    // Client 2 reads the history again
    let history = client2.get_history(channel2_id.channel_id).await?;

    println!("Client 2 history ---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }
    println!("---");

    // Client 2 sends a message
    client2.send_message(&channel2_info, "hello again!").await?;

    // Client 1 sends a message
    client1.send_message(&channel1_info, "how are you?").await?;

    // Client 1 reads the history
    let history = client1.get_history(channel2_id.channel_id).await?;

    println!("---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }
    println!("---");

    // Client 2 reads the history again
    let history = client2.get_history(channel2_id.channel_id).await?;

    println!("Client 2 history ---");
    for message in history {
        println!("{}", serde_json::to_string_pretty(&message)?);
    }
    println!("---");

    Ok(())
}
