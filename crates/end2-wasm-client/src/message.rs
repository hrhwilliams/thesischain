use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InboundChatMessage {
    pub message_id: Uuid,
    pub ciphertext: String, // b64encoded
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub is_pre_key: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct DecryptedMessage {
    pub message_id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub plaintext: String,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Serialize)]
pub struct MessagePayload {
    pub recipient_device_id: Uuid,
    pub ciphertext: String, // b64encoded
    pub is_pre_key: bool,
}
