use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{InboundMessagePayload, User, serialize_as_base64};

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_device_id: Uuid,
    pub created: OffsetDateTime,
    pub channel_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewChatMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_device_id: Uuid,
    pub channel_id: Uuid,
}

impl NewChatMessage {
    pub fn from_inbound(user: &User, message: &InboundChatMessage) -> Self {
        Self {
            id: message.message_id,
            sender_id: user.id,
            sender_device_id: message.device_id,
            channel_id: message.channel_id,
        }
    }
}

#[derive(Deserialize)]
pub struct InboundChatMessage {
    pub message_id: Uuid,
    pub device_id: Uuid,
    pub channel_id: Uuid,
    pub payloads: Vec<InboundMessagePayload>,
}

#[derive(Clone, Debug, Serialize, Queryable)]
pub struct OutboundChatMessage {
    pub message_id: Uuid,
    pub device_id: Uuid,
    #[serde(serialize_with = "serialize_as_base64")]
    pub ciphertext: Vec<u8>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    pub is_pre_key: bool,
}
