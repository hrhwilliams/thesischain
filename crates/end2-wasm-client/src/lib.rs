use std::{collections::HashMap, fmt, str::FromStr};

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, error::ComponentRange};
use uuid::Uuid;
use vodozemac::{
    Curve25519PublicKey,
    olm::{
        Account, AccountPickle, OlmMessage, PreKeyMessage, Session, SessionConfig, SessionPickle,
    },
};
use wasm_bindgen::prelude::*;

type UserId = Uuid;
type ChannelId = Uuid;
type DeviceId = Uuid;
type MessageId = Uuid;

pub struct Channel {
    // (device id, session)
    pub device_to_author: HashMap<DeviceId, UserId>,
    pub sessions: HashMap<DeviceId, Session>,
    pub x25519_keys: HashMap<DeviceId, Curve25519PublicKey>,
    pub message_history: Vec<DecryptedMessage>,
    pub unpublished_messages: HashMap<Uuid, UnpublishedMessage>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct DecryptedMessage {
    pub message_id: MessageId,
    pub author_id: UserId,
    pub plaintext: String,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

pub struct EncryptedMessage {
    pub device_id: DeviceId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub ciphertext: String,
}

#[derive(Deserialize)]
pub struct ChannelInfo {
    pub channel_id: ChannelId,
    pub users: Vec<User>,
    pub devices: Vec<Device>,
}

#[derive(Deserialize)]
pub struct MessageReceivedReply {
    pub message_id: Uuid,
    pub channel_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Deserialize)]
pub struct Device {
    pub device_id: DeviceId,
    pub user_id: UserId,
    pub ed25519: String,
    pub x25519: String,
    pub otk: Option<String>,
}

#[derive(Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub nickname: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct PickledChannel {
    pub device_to_author: HashMap<DeviceId, UserId>,
    pub sessions: HashMap<DeviceId, SessionPickle>,
    pub x25519_keys: HashMap<DeviceId, Curve25519PublicKey>,
    pub message_history: Vec<DecryptedMessage>,
    pub unpublished_messages: HashMap<Uuid, UnpublishedMessage>,
}

impl From<Channel> for PickledChannel {
    fn from(channel: Channel) -> Self {
        Self {
            device_to_author: channel.device_to_author,
            sessions: channel
                .sessions
                .into_iter()
                .map(|(id, session)| (id, session.pickle()))
                .collect(),
            x25519_keys: channel.x25519_keys.clone(),
            message_history: channel.message_history,
            unpublished_messages: channel.unpublished_messages,
        }
    }
}

impl From<PickledChannel> for Channel {
    fn from(pickle: PickledChannel) -> Self {
        Self {
            device_to_author: pickle.device_to_author,
            sessions: pickle
                .sessions
                .into_iter()
                .map(|(id, pickle)| (id, Session::from_pickle(pickle)))
                .collect(),
            x25519_keys: pickle.x25519_keys,
            message_history: pickle.message_history,
            unpublished_messages: pickle.unpublished_messages,
        }
    }
}

#[derive(Debug)]
pub enum DeviceError {
    UuidNotV7,
    TimeError(ComponentRange),
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<DeviceError> for JsError {
    fn from(value: DeviceError) -> Self {
        JsError::new(&value.to_string())
    }
}

#[derive(Serialize)]
struct IdentityKeys {
    x25519: String,
    ed25519: String,
    signature: String,
}

#[derive(Serialize)]
struct OneTimeKeys {
    otks: Vec<String>,
    signature: String,
}

#[derive(Deserialize)]
pub struct Otk {
    pub id: Uuid,
    pub device_id: Uuid,
    pub otk: String,
}

#[derive(Deserialize)]
pub struct InboundChatMessage {
    message_id: Uuid,
    device_id: Uuid,
    channel_id: Uuid,
    ciphertext: String, // b64encoded
    #[serde(with = "time::serde::rfc3339")]
    timestamp: OffsetDateTime,
    is_pre_key: bool,
}

#[derive(Serialize)]
pub struct OutboundChatMessage {
    pub message_id: Uuid,
    pub device_id: Uuid,
    pub channel_id: Uuid,
    pub payloads: Vec<OutboundChatMessagePayload>,
}

#[derive(Serialize)]
pub struct OutboundChatMessagePayload {
    pub recipient_device_id: Uuid,
    pub ciphertext: String, // b64encoded
    pub is_pre_key: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UnpublishedMessage {
    pub plaintext: String,
}

#[derive(Deserialize, Serialize)]
pub struct PickledDeviceContext {
    account: AccountPickle,
    channels: HashMap<Uuid, PickledChannel>,
    device_id: Uuid,
    user_id: Uuid,
}

#[wasm_bindgen]
pub struct DeviceContext {
    account: Account,
    channels: HashMap<Uuid, Channel>,
    device_id: Uuid,
    user_id: Uuid,
}

fn uuid_to_timestamp(uuid: Uuid) -> Result<OffsetDateTime, DeviceError> {
    let timestamp = uuid.get_timestamp().ok_or(DeviceError::UuidNotV7)?;
    let (seconds, ns) = timestamp.to_unix();

    OffsetDateTime::from_unix_timestamp(seconds as i64)
        .map_err(|e| DeviceError::TimeError(e))?
        .replace_nanosecond(ns)
        .map_err(|e| DeviceError::TimeError(e))
}

#[wasm_bindgen]
impl DeviceContext {
    pub fn new(user_id: &str, device_id: &str) -> Self {
        Self {
            account: Account::new(),
            channels: HashMap::default(),
            user_id: Uuid::parse_str(user_id).expect("device_id must be uuid"),
            device_id: Uuid::parse_str(device_id).expect("device_id must be uuid"),
        }
    }

    pub fn device_id(&self) -> String {
        self.device_id.to_string()
    }

    pub fn export_state(&self) -> Result<JsValue, JsError> {
        let pickled_state = PickledDeviceContext {
            device_id: self.device_id,
            user_id: self.user_id,
            account: self.account.pickle(),
            channels: self
                .channels
                .iter()
                .map(|(id, channel)| {
                    (
                        *id,
                        PickledChannel {
                            device_to_author: channel.device_to_author.clone(),
                            sessions: channel
                                .sessions
                                .iter()
                                .map(|(id, channel)| (*id, channel.pickle()))
                                .collect(),
                            message_history: channel.message_history.clone(),
                            x25519_keys: channel.x25519_keys.clone(),
                            unpublished_messages: channel.unpublished_messages.clone(),
                        },
                    )
                })
                .collect(),
        };

        Ok(serde_wasm_bindgen::to_value(
            &serde_json::to_string(&pickled_state).map_err(|e| JsError::new(&e.to_string()))?,
        )?)
    }

    pub fn try_from_state(state: &str) -> Result<Self, JsError> {
        let state = serde_json::from_str::<PickledDeviceContext>(state)?;

        Ok(Self {
            account: Account::from_pickle(state.account),
            channels: state
                .channels
                .into_iter()
                .map(|(id, channel)| (id, channel.into()))
                .collect(),
            device_id: state.device_id,
            user_id: state.user_id,
        })
    }

    pub fn get_identity_keys(&self) -> Result<JsValue, JsError> {
        let keys = self.account.identity_keys();
        let message = [keys.curve25519.as_bytes() as &[u8], keys.ed25519.as_bytes()].concat();

        let signature = self.account.sign(&message);

        Ok(serde_wasm_bindgen::to_value(&IdentityKeys {
            x25519: keys.curve25519.to_base64(),
            ed25519: keys.ed25519.to_base64(),
            signature: signature.to_base64(),
        })?)
    }

    pub fn generate_otks(&mut self, count: usize) -> Result<JsValue, JsError> {
        self.account.generate_one_time_keys(count);
        let account_otks = self.account.one_time_keys();
        let keys = account_otks.values().collect::<Vec<_>>();
        let message = keys
            .iter()
            .map(|&otk| otk.as_bytes() as &[u8])
            .collect::<Vec<&[u8]>>()
            .concat();
        let signature = self.account.sign(&message);

        let otks = keys.iter().map(|&k| k.to_base64()).collect();

        self.account.mark_keys_as_published();
        Ok(serde_wasm_bindgen::to_value(&OneTimeKeys {
            otks,
            signature: signature.to_base64(),
        })?)
    }

    pub fn has_channel_info_for(&self, channel_id: &str) -> Result<bool, JsError> {
        let channel_id = Uuid::from_str(channel_id).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(self.channels.contains_key(&channel_id))
    }

    pub fn create_session_from_otk(
        &mut self,
        channel_id: &str,
        otk: JsValue,
    ) -> Result<(), JsError> {
        let channel_id = Uuid::from_str(channel_id).map_err(|e| JsError::new(&e.to_string()))?;
        let otk: Otk =
            serde_wasm_bindgen::from_value(otk).map_err(|e| JsError::new(&e.to_string()))?;

        let otk_key = Curve25519PublicKey::from_base64(&otk.otk)?;
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(JsError::new("Missing channel"))?;
        let identity_key = channel
            .x25519_keys
            .get(&otk.device_id)
            .ok_or(JsError::new("Missing device"))?;

        let session = self.account.create_outbound_session(
            SessionConfig::version_2(),
            *identity_key,
            otk_key,
        );
        channel.sessions.insert(otk.device_id, session);

        Ok(())
    }

    // pub fn get_recipient_info(&self, channel_id: &str) -> Result<String, JsError> {
    //     let channel_id = Uuid::from_str(channel_id)?;
    //     Ok(self
    //         .channels
    //         .get(&channel_id)
    //         .and_then(|channel| Some(channel.their_username.clone()))
    //         .ok_or(JsError::new("No such session"))?)
    // }

    pub fn initialize_for_channel(&mut self, channel_info: JsValue) -> Result<(), JsError> {
        let channel_info = serde_wasm_bindgen::from_value::<ChannelInfo>(channel_info)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let device_to_author = channel_info
            .devices
            .iter()
            .filter(|&d| d.device_id != self.device_id)
            .map(|d| (d.device_id, d.user_id))
            .collect::<HashMap<Uuid, Uuid>>();
        let x25519_keys = channel_info
            .devices
            .iter()
            .filter(|&d| d.device_id != self.device_id)
            .map(|d| {
                let identity_key = Curve25519PublicKey::from_base64(&d.x25519)?;
                Ok((d.device_id, identity_key))
            })
            .collect::<Result<HashMap<Uuid, Curve25519PublicKey>, JsError>>()?;

        self.channels
            .entry(channel_info.channel_id)
            .and_modify(|e| {
                e.device_to_author = device_to_author.clone();
                e.x25519_keys = x25519_keys.clone();
            })
            .or_insert(Channel {
                device_to_author,
                sessions: HashMap::new(),
                x25519_keys,
                message_history: vec![],
                unpublished_messages: HashMap::new(),
            });

        Ok(())
    }

    // pub fn add_device_to_channel(&mut self, device: Device)

    pub fn missing_otks(&mut self, channel_id: &str) -> Result<JsValue, JsError> {
        let channel_id = Uuid::from_str(channel_id).map_err(|e| JsError::new(&e.to_string()))?;
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(JsError::new("Missing channel"))?;

        let missing_otks = channel
            .x25519_keys
            .keys()
            .filter(|&k| !channel.sessions.contains_key(k))
            .collect::<Vec<&Uuid>>();
        Ok(serde_wasm_bindgen::to_value(&missing_otks)?)
    }

    pub fn get_message_history(&self, channel_id: &str) -> Result<Vec<JsValue>, JsError> {
        let channel_id = Uuid::from_str(channel_id)?;
        let channel = self
            .channels
            .get(&channel_id)
            .ok_or(JsError::new("Missing channel"))?;

        channel
            .message_history
            .iter()
            .map(|m| {
                serde_wasm_bindgen::to_value(m)
                    .map_err(|e| JsError::new(&e.to_string()))
            })
            .collect()
    }

    pub fn encrypt(&mut self, channel_id: &str, message: &str) -> Result<JsValue, JsError> {
        let channel_id = Uuid::from_str(channel_id).map_err(|e| JsError::new(&e.to_string()))?;
        let message_id = Uuid::now_v7();
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(JsError::new("Missing channel"))?;

        let mut payloads = vec![];

        for (device_id, session) in channel.sessions.iter_mut() {
            if *device_id == self.device_id {
                continue;
            }

            let payload = match session.encrypt(message) {
                OlmMessage::Normal(m) => OutboundChatMessagePayload {
                    recipient_device_id: *device_id,
                    ciphertext: m.to_base64(),
                    is_pre_key: false,
                },
                OlmMessage::PreKey(pkm) => OutboundChatMessagePayload {
                    recipient_device_id: *device_id,
                    ciphertext: pkm.to_base64(),
                    is_pre_key: true,
                },
            };

            payloads.push(payload);
        }

        channel.unpublished_messages.insert(
            message_id,
            UnpublishedMessage {
                plaintext: message.to_string(),
            },
        );

        let outbound_message = OutboundChatMessage {
            message_id,
            channel_id,
            device_id: self.device_id,
            payloads,
        };

        Ok(serde_wasm_bindgen::to_value(&outbound_message)?)
    }

    pub fn message_received(&mut self, received: JsValue) -> Result<JsValue, JsError> {
        let received: MessageReceivedReply =
            serde_wasm_bindgen::from_value(received).map_err(|e| JsError::new(&e.to_string()))?;

        let channel = self
            .channels
            .get_mut(&received.channel_id)
            .ok_or(JsError::new("Missing channel"))?;

        let message = channel
            .unpublished_messages
            .remove(&received.message_id)
            .ok_or(JsError::new("Missing message"))?;
        let message = DecryptedMessage {
            message_id: received.message_id,
            author_id: self.user_id,
            plaintext: message.plaintext,
            timestamp: received.timestamp,
        };

        channel.message_history.push(message.clone());
        Ok(serde_wasm_bindgen::to_value(&message)?)
    }

    pub fn decrypt_new_session(&mut self, message: JsValue) -> Result<JsValue, JsError> {
        let message: InboundChatMessage =
            serde_wasm_bindgen::from_value(message).map_err(|e| JsError::new(&e.to_string()))?;

        let channel = self
            .channels
            .get_mut(&message.channel_id)
            .ok_or(JsError::new("Missing channel"))?;

        if message.device_id == self.device_id {
            return Err(JsError::new("Cannot decrypt message from self"));
        }

        if message.is_pre_key {
            let identity_key = channel
                .x25519_keys
                .get(&message.device_id)
                .ok_or(JsError::new("Missing device"))?;
            let author_id = channel
                .device_to_author
                .get(&message.device_id)
                .ok_or(JsError::new("Missing device"))?;

            let pkm = PreKeyMessage::from_base64(&message.ciphertext)?;
            let result = self.account.create_inbound_session(*identity_key, &pkm)?;
            let plaintext = String::from_utf8(result.plaintext)?;

            channel.sessions.insert(message.device_id, result.session);
            let message = DecryptedMessage {
                message_id: message.message_id,
                author_id: *author_id,
                plaintext,
                timestamp: message.timestamp,
            };

            channel.message_history.push(message.clone());
            Ok(serde_wasm_bindgen::to_value(&message)?)
        } else {
            Err(JsError::new("Expected PreKeyMessage"))
        }
    }

    pub fn decrypt(&mut self, message: JsValue) -> Result<JsValue, JsError> {
        let message: InboundChatMessage =
            serde_wasm_bindgen::from_value(message).map_err(|e| JsError::new(&e.to_string()))?;

        if message.device_id == self.device_id {
            return Err(JsError::new("Cannot decrypt message from self"));
        }

        if !message.is_pre_key {
            let msg =
                OlmMessage::from_parts(1, &BASE64_STANDARD_NO_PAD.decode(&message.ciphertext)?)?;
            let channel = self
                .channels
                .get_mut(&message.channel_id)
                .ok_or(JsError::new("Missing channel"))?;
            let author_id = channel
                .device_to_author
                .get(&message.device_id)
                .ok_or(JsError::new("Missing device"))?;
            let session = channel
                .sessions
                .get_mut(&message.device_id)
                .ok_or(JsError::new("Missing session"))?;

            let plaintext = session.decrypt(&msg)?;
            let plaintext = String::from_utf8(plaintext)?;
            let message = DecryptedMessage {
                message_id: message.message_id,
                author_id: *author_id,
                plaintext,
                timestamp: message.timestamp,
            };

            channel.message_history.push(message.clone());
            Ok(serde_wasm_bindgen::to_value(&message)?)
        } else {
            Err(JsError::new("Expected normal message"))
        }
    }
}

#[cfg(test)]
mod tests {
    use time::Duration;

    use super::*;

    #[test]
    fn test_uuid_to_time_offset() {
        let now = OffsetDateTime::now_utc();
        let uuid = Uuid::now_v7();
        let time_offset = uuid_to_timestamp(uuid).expect("Failed to convert to timestamp");
        assert!(time_offset - now < Duration::milliseconds(1_000));
    }
}
