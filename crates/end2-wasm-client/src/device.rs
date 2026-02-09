use std::str::FromStr;

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

#[derive(Serialize)]
pub struct EncryptionOutput {
    pub session: SessionPickle,
    pub payload: MessagePayload,
}

#[derive(Serialize)]
pub struct DecryptionOutput {
    pub session: SessionPickle,
    pub payload: DecryptedMessage,
}

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
}

impl From<PickledDevice> for Device {
    fn from(pickle: PickledDevice) -> Self {
        Self {
            account: Account::from_pickle(pickle.account),
            device_id: pickle.device_id,
        }
    }
}

#[wasm_bindgen]
pub struct Device {
    account: Account,
    device_id: Uuid,
}

#[wasm_bindgen]
#[allow(clippy::unused_self)]
impl Device {
    /// Creates a new device with the given ID.
    ///
    /// # Errors
    /// Returns `JsError` if the device ID is not a valid UUID.
    pub fn new(device_id: &str) -> Result<Self, JsError> {
        Ok(Self {
            device_id: Uuid::from_str(device_id)?,
            account: Account::new(),
        })
    }

    #[must_use]
    pub fn device_id(&self) -> String {
        self.device_id.to_string()
    }

    /// Serializes the device state for storage.
    ///
    /// # Errors
    /// Returns `JsError` if serialization fails.
    pub fn to_pickle(&self) -> Result<JsValue, JsError> {
        let pickle = PickledDevice {
            account: self.account.pickle(),
            device_id: self.device_id,
        };

        Ok(serde_wasm_bindgen::to_value(&pickle)?)
    }

    /// Restores a device from a previously pickled state.
    ///
    /// # Errors
    /// Returns `JsError` if deserialization fails.
    pub fn try_from_pickle(state: JsValue) -> Result<Self, JsError> {
        let state = serde_wasm_bindgen::from_value::<PickledDevice>(state)?;
        Ok(Self::from(state))
    }

    /// Returns the device's signed identity keys.
    ///
    /// # Errors
    /// Returns `JsError` if serialization fails.
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

    /// Checks whether a session needs a one-time key for initial message.
    ///
    /// # Errors
    /// Returns `JsError` if the pickle is invalid.
    pub fn needs_otk(&self, pickle: JsValue) -> Result<bool, JsError> {
        let pickle = serde_wasm_bindgen::from_value::<SessionPickle>(pickle)?;
        let session = Session::from(pickle);
        Ok(!session.has_received_message())
    }

    /// Generates one-time keys with signed upload payload.
    ///
    /// # Errors
    /// Returns `JsError` if serialization fails.
    pub fn gen_otks(&mut self, mut count: usize) -> Result<JsValue, JsError> {
        if count > self.account.max_number_of_one_time_keys() {
            count = self.account.max_number_of_one_time_keys();
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

    /// Encrypts a plaintext message using an existing session.
    ///
    /// # Errors
    /// Returns `JsError` if the session or encryption fails.
    pub fn encrypt(
        &self,
        pickle: JsValue,
        device: JsValue,
        plaintext: &str,
    ) -> Result<JsValue, JsError> {
        let pickle = serde_wasm_bindgen::from_value::<SessionPickle>(pickle)?;
        let device = serde_wasm_bindgen::from_value::<DeviceInfo>(device)?;
        let mut session = Session::from_pickle(pickle);

        let OlmMessage::Normal(msg) = session.encrypt(plaintext) else {
            return Err(JsError::new("expected Normal message"));
        };

        let payload = MessagePayload {
            recipient_device_id: device.device_id,
            ciphertext: msg.to_base64(),
            is_pre_key: false,
        };

        let session = session.pickle();

        let output = EncryptionOutput { session, payload };

        Ok(serde_wasm_bindgen::to_value(&output)?)
    }

    /// Encrypts a plaintext message using a one-time key to establish a new session.
    ///
    /// # Errors
    /// Returns `JsError` if the OTK is invalid or encryption fails.
    pub fn encrypt_otk(
        &self,
        device: JsValue,
        plaintext: &str,
        otk: &str,
    ) -> Result<JsValue, JsError> {
        let device: DeviceInfo = serde_wasm_bindgen::from_value(device)?;
        let otk = Curve25519PublicKey::from_base64(otk)?;
        let identity_key = Curve25519PublicKey::from_base64(&device.x25519)?;
        let mut session =
            self.account
                .create_outbound_session(SessionConfig::version_2(), identity_key, otk);
        let OlmMessage::PreKey(pkm) = session.encrypt(plaintext) else {
            return Err(JsError::new("expected PreKey message"));
        };

        let payload = MessagePayload {
            recipient_device_id: device.device_id,
            ciphertext: pkm.to_base64(),
            is_pre_key: true,
        };

        let output = EncryptionOutput {
            session: session.pickle(),
            payload,
        };

        Ok(serde_wasm_bindgen::to_value(&output)?)
    }

    /// Decrypts a normal (non-pre-key) message using an existing session.
    ///
    /// # Errors
    /// Returns `JsError` if the session or decryption fails.
    pub fn decrypt(
        &self,
        pickle: JsValue,
        device: JsValue,
        payload: JsValue,
    ) -> Result<JsValue, JsError> {
        let pickle = serde_wasm_bindgen::from_value::<SessionPickle>(pickle)?;
        let device: DeviceInfo = serde_wasm_bindgen::from_value(device)?;
        let payload: InboundChatMessage = serde_wasm_bindgen::from_value(payload)?;
        let mut session = Session::from_pickle(pickle);

        let msg = Message::from_base64(&payload.ciphertext)?;
        let plaintext_bytes = session.decrypt(&OlmMessage::Normal(msg))?;

        let plaintext = String::from_utf8(plaintext_bytes)?;

        let decrypted = DecryptedMessage {
            message_id: payload.message_id,
            channel_id: payload.channel_id,
            author_id: device.user_id,
            plaintext,
            timestamp: payload.timestamp,
        };

        let output = DecryptionOutput {
            session: session.pickle(),
            payload: decrypted,
        };

        Ok(serde_wasm_bindgen::to_value(&output)?)
    }

    /// Decrypts a pre-key message, establishing a new inbound session.
    ///
    /// # Errors
    /// Returns `JsError` if the pre-key message is invalid or decryption fails.
    pub fn decrypt_otk(&mut self, device: JsValue, payload: JsValue) -> Result<JsValue, JsError> {
        let device: DeviceInfo = serde_wasm_bindgen::from_value(device)?;
        let payload: InboundChatMessage = serde_wasm_bindgen::from_value(payload)?;

        let identity_key = Curve25519PublicKey::from_base64(&device.x25519)?;
        let pkm = PreKeyMessage::from_base64(&payload.ciphertext)?;

        let result = self.account.create_inbound_session(identity_key, &pkm)?;
        let plaintext = String::from_utf8(result.plaintext)?;

        let decrypted = DecryptedMessage {
            message_id: payload.message_id,
            channel_id: payload.channel_id,
            author_id: device.user_id,
            plaintext,
            timestamp: payload.timestamp,
        };

        let output = DecryptionOutput {
            session: result.session.pickle(),
            payload: decrypted,
        };

        Ok(serde_wasm_bindgen::to_value(&output)?)
    }
}
