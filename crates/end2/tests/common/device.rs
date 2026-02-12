use anyhow::Result;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use vodozemac::{
    Curve25519PublicKey,
    olm::{Account, Message, OlmMessage, PreKeyMessage, Session, SessionConfig},
};

#[derive(Serialize)]
pub struct IdentityKeys {
    x25519: String,
    ed25519: String,
    signature: String,
}

#[derive(Serialize)]
pub struct UploadOtks {
    created: Vec<String>,
    removed: Vec<String>,
    created_signature: String,
    removed_signature: Option<String>,
}

#[derive(Deserialize)]
pub struct Otk {
    pub id: Uuid,
    pub device_id: String,
    pub otk: String,
}

#[derive(Serialize)]
pub struct Otks2 {
    created: Vec<String>,
    removed: Vec<String>,
    created_signature: String,
    removed_signature: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeviceInfo {
    pub device_id: Uuid,
    pub user_id: Uuid,
    pub x25519: Option<String>,
    pub ed25519: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessagePayload {
    pub recipient_device_id: Uuid,
    pub ciphertext: String,
    pub is_pre_key: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DecryptedMessage {
    pub message_id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub plaintext: String,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct InboundChatMessage {
    pub message_id: Uuid,
    pub device_id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub ciphertext: String,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub is_pre_key: bool,
}

pub struct Device {
    account: Account,
    device_id: Uuid,
}

impl Device {
    pub fn new(device_id: Uuid) -> Self {
        Self {
            account: Account::new(),
            device_id,
        }
    }

    pub fn id(&self) -> Uuid {
        self.device_id
    }

    pub fn x25519_public_key_bytes(&self) -> [u8; 32] {
        *self.account.identity_keys().curve25519.as_bytes()
    }

    pub fn ed25519_public_key_bytes(&self) -> [u8; 32] {
        *self.account.identity_keys().ed25519.as_bytes()
    }

    pub fn get_identity_keys(&self) -> IdentityKeys {
        let keys = self.account.identity_keys();
        let message = [keys.curve25519.as_bytes() as &[u8], keys.ed25519.as_bytes()].concat();

        let signature = self.account.sign(&message);

        IdentityKeys {
            x25519: keys.curve25519.to_base64(),
            ed25519: keys.ed25519.to_base64(),
            signature: signature.to_base64(),
        }
    }

    pub fn get_otks(&mut self, mut count: usize) -> UploadOtks {
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

        payload
    }

    pub fn encrypt(
        &self,
        mut session: Session,
        device: &DeviceInfo,
        plaintext: &str,
    ) -> Result<(Session, MessagePayload)> {
        let OlmMessage::Normal(msg) = session.encrypt(plaintext) else {
            return Err(anyhow::anyhow!("expected Normal message"));
        };

        let payload = MessagePayload {
            recipient_device_id: device.device_id,
            ciphertext: msg.to_base64(),
            is_pre_key: false,
        };

        Ok((session, payload))
    }

    pub fn encrypt_otk(
        &self,
        device: &DeviceInfo,
        plaintext: &str,
        otk: Curve25519PublicKey,
    ) -> Result<(Session, MessagePayload)> {
        let identity_key = Curve25519PublicKey::from_base64(device.x25519.as_ref().unwrap())?;
        let mut session =
            self.account
                .create_outbound_session(SessionConfig::version_2(), identity_key, otk);
        let OlmMessage::PreKey(pkm) = session.encrypt(plaintext) else {
            return Err(anyhow::anyhow!("expected PreKey message"));
        };

        let payload = MessagePayload {
            recipient_device_id: device.device_id,
            ciphertext: pkm.to_base64(),
            is_pre_key: true,
        };

        Ok((session, payload))
    }

    pub fn decrypt(
        &mut self,
        mut session: Session,
        device: &DeviceInfo,
        payload: InboundChatMessage,
    ) -> Result<(Session, DecryptedMessage)> {
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

        Ok((session, decrypted))
    }

    pub fn decrypt_otk(
        &mut self,
        device: &DeviceInfo,
        payload: InboundChatMessage,
    ) -> Result<(Session, DecryptedMessage)> {
        let identity_key = Curve25519PublicKey::from_base64(device.x25519.as_ref().unwrap())?;
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

        Ok((result.session, decrypted))
    }
}
