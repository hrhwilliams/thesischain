use std::collections::HashMap;

use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use time::OffsetDateTime;
use uuid::Uuid;
use vodozemac::{
    Curve25519PublicKey,
    olm::{
        Account, AccountPickle, MessageType, OlmMessage, PreKeyMessage, Session, SessionConfig,
        SessionPickle,
    },
};
use wasm_bindgen::prelude::*;

#[derive(Deserialize, Serialize)]
pub struct NewUser {
    pub username: String,
    pub ed25519: String,
    pub curve25519: String,
    pub signature: String,
}

#[derive(Deserialize)]
struct ChallengeInput {
    id: String,
    user_id: String,
}

#[derive(Deserialize)]
pub struct InboundChatMessage {
    id: Uuid,
    author: String,
    author_id: Uuid,
    channel_id: Uuid,
    content: String, // b64encoded
    pre_key: bool,
}

#[derive(Serialize)]
pub struct OutboundChatMessage {
    channel_id: Uuid,
    content: String, // b64encoded
    pre_key: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Message {
    author: String,
    plaintext: String,
    #[serde(with = "time::serde::rfc3339")]
    timestamp: OffsetDateTime,
}

pub struct Channel {
    their_username: String,
    their_identity_key: Curve25519PublicKey,
    session: Session,
    message_history: Vec<Message>,
}

#[derive(Deserialize, Serialize)]
pub struct PickledChannel {
    their_username: String,
    their_identity_key: String,
    session_pickle: SessionPickle,
    message_history: Vec<Message>,
}

#[derive(Deserialize, Serialize)]
pub struct PickledClient {
    account: AccountPickle,
    sessions: HashMap<Uuid, PickledChannel>,
}

#[wasm_bindgen]
pub struct End2ClientSession {
    account: Account,
    sessions: HashMap<Uuid, Channel>,
}

fn uuid_to_timestamp(uuid: Uuid) -> Result<OffsetDateTime, JsError> {
    let timestamp = uuid.get_timestamp().ok_or(JsError::new("UUID not v7"))?;
    let (seconds, ns) = timestamp.to_unix();
    let tm = OffsetDateTime::from_unix_timestamp(seconds as i64)
        .map_err(|e| JsError::new(&e.to_string()))?
        .replace_nanosecond(ns)
        .map_err(|e| JsError::new(&e.to_string()))?;
    Ok(tm)
}

#[wasm_bindgen]
impl End2ClientSession {
    pub fn new() -> Self {
        Self {
            account: Account::new(),
            sessions: HashMap::default(),
        }
    }

    pub fn export_state(&self) -> Result<JsValue, JsError> {
        let mut pickled_channels = HashMap::new();
        for (channel_id, channel) in &self.sessions {
            pickled_channels.insert(
                *channel_id,
                PickledChannel {
                    their_username: channel.their_username.clone(),
                    their_identity_key: channel.their_identity_key.to_base64(),
                    session_pickle: channel.session.pickle(),
                    message_history: channel.message_history.clone(),
                },
            );
        }

        let pickled_state = PickledClient {
            account: self.account.pickle(),
            sessions: pickled_channels,
        };

        Ok(serde_wasm_bindgen::to_value(
            &serde_json::to_string(&pickled_state).map_err(|e| JsError::new(&e.to_string()))?,
        )?)
    }

    pub fn try_from_state(state: &str) -> Result<Self, JsError> {
        let state = serde_json::from_str::<PickledClient>(state)?;

        let mut sessions = HashMap::new();
        for (channel_id, channel) in state.sessions {
            sessions.insert(
                channel_id,
                Channel {
                    their_username: channel.their_username,
                    their_identity_key: Curve25519PublicKey::from_base64(
                        &channel.their_identity_key,
                    )?,
                    session: Session::from_pickle(channel.session_pickle),
                    message_history: channel.message_history,
                },
            );
        }

        Ok(Self {
            account: Account::from_pickle(state.account),
            sessions,
        })
    }

    pub fn get_identity_keys(&self) -> Result<JsValue, JsError> {
        let keys = self.account.identity_keys();
        Ok(serde_wasm_bindgen::to_value(&keys)?)
    }

    pub fn generate_otks(&mut self, count: usize) -> Result<JsValue, JsError> {
        self.account.generate_one_time_keys(count);
        let otks = self
            .account
            .one_time_keys()
            .values()
            .map(|otk| otk.to_base64())
            .collect::<Vec<String>>();
        self.account.mark_keys_as_published();
        Ok(serde_wasm_bindgen::to_value(&otks)?)
    }

    pub fn channel_has_session(&self, channel_id: &str) -> Result<bool, JsError> {
        let channel_id = Uuid::from_str(channel_id)?;
        Ok(self.sessions.contains_key(&channel_id))
    }

    pub fn get_recipient_info(&self, channel_id: &str) -> Result<String, JsError> {
        let channel_id = Uuid::from_str(channel_id)?;
        Ok(self
            .sessions
            .get(&channel_id)
            .and_then(|channel| Some(channel.their_username.clone()))
            .ok_or(JsError::new("No such session"))?)
    }

    pub fn create_outbound_session(
        &mut self,
        channel_id: &str,
        their_username: &str,
        identity_key: &str,
        one_time_key: &str,
    ) -> Result<(), JsError> {
        let channel_id = Uuid::from_str(channel_id)?;
        let identity_key = Curve25519PublicKey::from_base64(identity_key)?;
        let one_time_key = Curve25519PublicKey::from_base64(one_time_key)?;
        let session = self.account.create_outbound_session(
            SessionConfig::version_2(),
            identity_key,
            one_time_key,
        );

        self.sessions.insert(
            channel_id,
            Channel {
                their_username: their_username.to_string(),
                their_identity_key: identity_key,
                session,
                message_history: vec![],
            },
        );

        Ok(())
    }

    pub fn encrypt(&mut self, channel_id: &str, message: &str) -> Result<JsValue, JsError> {
        let channel_id = Uuid::from_str(channel_id).map_err(|e| JsError::new(&e.to_string()))?;
        let channel = self
            .sessions
            .get_mut(&channel_id)
            .ok_or(JsError::new("Missing session"))?;

        // let olm_message = channel.session.encrypt(message);
        let outbound_message = match channel.session.encrypt(message) {
            OlmMessage::Normal(m) => OutboundChatMessage {
                channel_id,
                content: m.to_base64(),
                pre_key: false
            },
            OlmMessage::PreKey(pkm) => OutboundChatMessage {
                channel_id,
                content: pkm.to_base64(),
                pre_key: true
            },
        };

        Ok(serde_wasm_bindgen::to_value(&outbound_message)?)
    }

    pub fn decrypt_new_session(
        &mut self,
        channel_id: &str,
        their_username: &str,
        identity_key: &str,
        message: &str,
    ) -> Result<String, JsError> {
        let channel_id = Uuid::from_str(channel_id).map_err(|e| JsError::new(&e.to_string()))?;
        let identity_key = Curve25519PublicKey::from_base64(identity_key)?;
        let message = serde_json::from_str::<InboundChatMessage>(message)?;

        if message.pre_key {
            let pkm = PreKeyMessage::from_base64(&message.content)?;
            let result = self.account.create_inbound_session(identity_key, &pkm)?;
            let plaintext = String::from_utf8(result.plaintext)?;
            self.sessions.insert(
                channel_id,
                Channel {
                    their_username: their_username.to_string(),
                    their_identity_key: identity_key,
                    session: result.session,
                    message_history: vec![Message {
                        author: message.author,
                        plaintext: plaintext.clone(),
                        timestamp: uuid_to_timestamp(message.id)?,
                    }],
                },
            );

            Ok(plaintext)
        } else {
            Err(JsError::new("Expected PreKeyMessage"))
        }
    }

    pub fn decrypt(&mut self, channel_id: &str, message: &str) -> Result<String, JsError> {
        let channel_id = Uuid::from_str(channel_id)?;
        let message = serde_json::from_str::<InboundChatMessage>(message)?;

        if !message.pre_key {
            let msg = OlmMessage::from_parts(1, &BASE64_STANDARD_NO_PAD.decode(&message.content)?)?;
            let channel = self
                .sessions
                .get_mut(&channel_id)
                .ok_or(JsError::new("Missing session"))?;
            let plaintext = channel.session.decrypt(&msg)?;
            let plaintext = String::from_utf8(plaintext)?;
            channel.message_history.push(Message {
                author: message.author,
                plaintext: plaintext.clone(),
                timestamp: uuid_to_timestamp(message.id)?,
            });
            Ok(plaintext)
        } else {
            Err(JsError::new("Expected normal message"))
        }
    }

    // pub fn get_message_history(&self, channel_id: &str) {
    //     let channel_id = Uuid::from_str(channel_id).map_err(|e| JsError::new(&e.to_string()))?;
    // }

    pub fn register(&self, username: String) -> Result<JsValue, JsError> {
        let identity_keys = self.account.identity_keys();
        let ed25519 = identity_keys.ed25519;
        let curve25519 = identity_keys.curve25519;

        let message = [
            username.as_bytes(),
            curve25519.as_bytes(),
            ed25519.as_bytes(),
        ]
        .concat();

        let signature = self.account.sign(&message);

        let new_user = NewUser {
            username,
            ed25519: ed25519.to_base64(),
            curve25519: curve25519.to_base64(),
            signature: signature.to_base64(),
        };

        Ok(serde_wasm_bindgen::to_value(&new_user)?)
    }

    pub fn sign_challenge(&self, challenge: JsValue) -> Result<JsValue, JsError> {
        let input: ChallengeInput = serde_wasm_bindgen::from_value(challenge)?;
        let id = Uuid::try_parse(&input.id).unwrap();
        let user_id = Uuid::try_parse(&input.user_id).unwrap();
        let message = [id.into_bytes(), user_id.into_bytes()].concat();

        let signature_bytes = self.account.sign(&message);

        Ok(serde_wasm_bindgen::to_value(&signature_bytes.to_base64())?)
    }
}

#[wasm_bindgen]
pub fn add_rs(a: i32, b: i32) -> i32 {
    a + b
}
