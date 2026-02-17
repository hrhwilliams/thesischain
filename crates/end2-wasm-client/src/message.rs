use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::types::{ChannelId, DeviceId, MessageId, UserId};

#[derive(Deserialize)]
pub struct InboundChatMessage {
    pub message_id: MessageId,
    pub device_id: DeviceId,
    pub channel_id: ChannelId,
    pub ciphertext: String, // b64encoded
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub is_pre_key: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct DecryptedMessage {
    pub message_id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub plaintext: String,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Serialize)]
pub struct MessagePayload {
    pub recipient_device_id: DeviceId,
    pub ciphertext: String, // b64encoded
    pub is_pre_key: bool,
}
