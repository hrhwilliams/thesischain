use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vodozemac::{
    Curve25519PublicKey,
    olm::{
        Account, AccountPickle, Message, OlmMessage, PreKeyMessage, Session, SessionConfig,
        SessionPickle,
    },
};
use wasm_bindgen::prelude::*;

use crate::message::{DecryptedMessage, InboundChatMessage, MessagePayload};

#[derive(Deserialize)]
pub struct DeviceInfo {
    pub device_id: Uuid,
    pub user_id: Uuid,
    pub ed25519: String,
    pub x25519: String,
}

#[derive(Serialize)]
pub struct UploadOtks {
    created: Vec<String>,
    removed: Vec<String>,
    created_signature: String,
    removed_signature: Option<String>,
}

#[derive(Serialize)]
pub struct IdentityKeys {
    pub device_id: Uuid,
    pub x25519: String,
    pub ed25519: String,
    pub signature: String,
}

#[derive(Deserialize, Serialize)]
pub struct PickledDevice {
    account: AccountPickle,
    device_id: Uuid,
    sessions: HashMap<Uuid, HashMap<Uuid, SessionPickle>>,
}

impl From<PickledDevice> for Device {
    fn from(pickle: PickledDevice) -> Self {
        Self {
            account: Account::from_pickle(pickle.account),
            device_id: pickle.device_id,
            sessions: pickle
                .sessions
                .into_iter()
                .map(|(channel_id, device_map)| {
                    let inner = device_map
                        .into_iter()
                        .map(|(device_id, session_pickle)| {
                            (device_id, Session::from_pickle(session_pickle))
                        })
                        .collect();

                    (channel_id, inner)
                })
                .collect(),
        }
    }
}

#[wasm_bindgen]
pub struct Device {
    account: Account,
    device_id: Uuid,
    // channel_id -> device_id -> session
    sessions: HashMap<Uuid, HashMap<Uuid, Session>>,
}

#[wasm_bindgen]
impl Device {
    pub fn new(device_id: &str) -> Result<Self, JsError> {
        Ok(Self {
            device_id: Uuid::from_str(device_id)?,
            account: Account::new(),
            sessions: HashMap::default(),
        })
    }

    pub fn device_id(&self) -> String {
        self.device_id.to_string()
    }

    pub fn to_pickle(&self) -> Result<JsValue, JsError> {
        let pickle = PickledDevice {
            account: self.account.pickle(),
            device_id: self.device_id,
            sessions: self
                .sessions
                .iter()
                .map(|(channel_id, device_map)| {
                    let inner = device_map
                        .iter()
                        .map(|(device_id, session)| (*device_id, session.pickle()))
                        .collect();
                    (*channel_id, inner)
                })
                .collect(),
        };

        Ok(serde_wasm_bindgen::to_value(&pickle)?)
    }

    pub fn try_from_pickle(state: JsValue) -> Result<Self, JsError> {
        let state = serde_wasm_bindgen::from_value::<PickledDevice>(state)?;
        Ok(Self::from(state))
    }

    pub fn keys(&self) -> Result<JsValue, JsError> {
        let keys = self.account.identity_keys();
        let message = [
            keys.curve25519.as_bytes() as &[u8],
            keys.ed25519.as_bytes() as &[u8],
        ]
        .concat();
        let signature = self.account.sign(&message);

        let payload = IdentityKeys {
            device_id: self.device_id,
            x25519: keys.curve25519.to_base64(),
            ed25519: keys.ed25519.to_base64(),
            signature: signature.to_base64(),
        };

        Ok(serde_wasm_bindgen::to_value(&payload)?)
    }

    pub fn need_otk(&self, channel_id: &str, device_id: &str) -> Result<bool, JsError> {
        let channel_id = Uuid::from_str(channel_id)?;
        let device_id = Uuid::from_str(device_id)?;

        Ok(self
            .sessions
            .get(&channel_id)
            .and_then(|channel_map| channel_map.get(&device_id))
            .is_some_and(|session| !session.has_received_message()))
    }

    pub fn gen_otks(&mut self, mut count: usize) -> Result<JsValue, JsError> {
        if count > self.account.max_number_of_one_time_keys() {
            count = self.account.max_number_of_one_time_keys()
        }

        let otks = self.account.generate_one_time_keys(count);
        self.account.mark_keys_as_published();

        let created_concat = otks
            .created
            .iter()
            .map(|k| k.as_bytes() as &[u8])
            .collect::<Vec<&[u8]>>()
            .concat();
        let removed_concat = otks
            .removed
            .iter()
            .map(|k| k.as_bytes() as &[u8])
            .collect::<Vec<&[u8]>>()
            .concat();

        let created_signature = self.account.sign(created_concat);
        let removed_signature = if removed_concat.is_empty() {
            None
        } else {
            Some(self.account.sign(removed_concat))
        };

        let payload = UploadOtks {
            created: otks.created.into_iter().map(|k| k.to_base64()).collect(),
            removed: otks.removed.into_iter().map(|k| k.to_base64()).collect(),
            created_signature: created_signature.to_base64(),
            removed_signature: removed_signature.map(|k| k.to_base64()),
        };

        Ok(serde_wasm_bindgen::to_value(&payload)?)
    }

    pub fn encrypt(
        &mut self,
        channel_id: &str,
        device_id: &str,
        plaintext: &str,
    ) -> Result<JsValue, JsError> {
        let channel_id = Uuid::from_str(channel_id)?;
        let device_id = Uuid::from_str(device_id)?;
        let session = self
            .sessions
            .get_mut(&channel_id)
            .and_then(|channel_map| channel_map.get_mut(&device_id))
            .ok_or_else(|| JsError::new("missing session"))?;
        let OlmMessage::Normal(msg) = session.encrypt(plaintext) else {
            return Err(JsError::new("expected PreKey message"));
        };

        let payload = MessagePayload {
            recipient_device_id: device_id,
            ciphertext: msg.to_base64(),
            is_pre_key: false,
        };

        Ok(serde_wasm_bindgen::to_value(&payload)?)
    }

    pub fn encrypt_otk(
        &mut self,
        channel_id: &str,
        device: JsValue,
        plaintext: &str,
        otk: &str,
    ) -> Result<JsValue, JsError> {
        let channel_id = Uuid::from_str(channel_id)?;
        let device: DeviceInfo = serde_wasm_bindgen::from_value(device)?;
        let otk = Curve25519PublicKey::from_base64(otk)?;
        let identity_key = Curve25519PublicKey::from_base64(&device.x25519)?;

        let mut session =
            self.account
                .create_outbound_session(SessionConfig::version_2(), identity_key, otk);
        let OlmMessage::PreKey(pkm) = session.encrypt(plaintext) else {
            return Err(JsError::new("expected PreKey message"));
        };

        self.sessions
            .entry(channel_id)
            .or_default()
            .insert(device.device_id, session);

        let payload = MessagePayload {
            recipient_device_id: device.device_id,
            ciphertext: pkm.to_base64(),
            is_pre_key: true,
        };

        Ok(serde_wasm_bindgen::to_value(&payload)?)
    }

    pub fn decrypt(
        &mut self,
        channel_id: &str,
        device: JsValue,
        payload: JsValue,
    ) -> Result<JsValue, JsError> {
        let channel_id = Uuid::from_str(channel_id)?;
        let device: DeviceInfo = serde_wasm_bindgen::from_value(device)?;
        let payload: InboundChatMessage = serde_wasm_bindgen::from_value(payload)?;

        let plaintext = if payload.is_pre_key {
            let pkm = PreKeyMessage::from_base64(&payload.ciphertext)?;
            let identity_key = Curve25519PublicKey::from_base64(&device.x25519)?;
            let result = self.account.create_inbound_session(identity_key, &pkm)?;
            self.sessions
                .entry(channel_id)
                .or_default()
                .insert(device.device_id, result.session);

            String::from_utf8(result.plaintext)?
        } else {
            let session = self
                .sessions
                .get_mut(&channel_id)
                .and_then(|channel_map| channel_map.get_mut(&device.device_id))
                .ok_or_else(|| JsError::new("missing session"))?;
            let msg = Message::from_base64(&payload.ciphertext)?;
            let plaintext_bytes = session.decrypt(&OlmMessage::Normal(msg))?;

            String::from_utf8(plaintext_bytes)?
        };

        let decrypted = DecryptedMessage {
            message_id: payload.message_id,
            channel_id,
            author_id: device.user_id,
            plaintext,
            timestamp: payload.timestamp,
        };

        Ok(serde_wasm_bindgen::to_value(&decrypted)?)
    }
}
