use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::{ChannelId, DeviceId, MessageId, UserId};

use crate::{InboundMessagePayload, User, serialize_as_base64};

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatMessage {
    pub id: MessageId,
    pub sender_id: UserId,
    pub sender_device_id: DeviceId,
    pub created: OffsetDateTime,
    pub channel_id: ChannelId,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewChatMessage {
    pub id: MessageId,
    pub sender_id: UserId,
    pub sender_device_id: DeviceId,
    pub channel_id: ChannelId,
}

impl NewChatMessage {
    #[must_use]
    pub const fn from_inbound(user: &User, message: &InboundChatMessage) -> Self {
        Self {
            id: message.message_id,
            sender_id: user.id,
            sender_device_id: message.device_id,
            channel_id: message.channel_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct InboundChatMessage {
    pub message_id: MessageId,
    pub device_id: DeviceId,
    pub channel_id: ChannelId,
    pub payloads: Vec<InboundMessagePayload>,
}

#[derive(Clone, Debug, Serialize, Queryable)]
pub struct OutboundChatMessage {
    pub message_id: MessageId,
    pub device_id: DeviceId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
    #[serde(serialize_with = "serialize_as_base64")]
    pub ciphertext: Vec<u8>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub is_pre_key: bool,
}
