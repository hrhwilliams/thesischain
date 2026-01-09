use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{InboundMessagePayload, serialize_as_base64};

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_device: Uuid,
    pub channel_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewChatMessage {
    pub sender_id: Uuid,
    pub sender_device: Uuid,
    pub channel_id: Uuid,
}

impl From<InboundChatMessage> for NewChatMessage {
    fn from(msg: InboundChatMessage) -> Self {
        Self {
            sender_id: msg.sender_id,
            sender_device: msg.sender_device,
            channel_id: msg.channel_id,
        }
    }
}

#[derive(Deserialize)]
pub struct InboundChatMessage {
    pub sender_id: Uuid,
    pub sender_device: Uuid,
    pub channel_id: Uuid,
    pub payloads: Vec<InboundMessagePayload>,
}

#[derive(Serialize, Queryable)]
pub struct OutboundChatMessage {
    pub id: Uuid,
    pub author_id: Uuid,
    pub author_username: String,
    pub author_nickname: Option<String>,
    #[serde(serialize_with = "serialize_as_base64")]
    pub ciphertext: Vec<u8>,
    pub is_pre_key: bool,
}
