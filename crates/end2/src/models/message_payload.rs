use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppError;

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::message_payload)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MessagePayload {
    pub message_id: Uuid,
    pub recipient_device: Uuid,
    pub ciphertext: Vec<u8>,
    pub is_pre_key: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::message_payload)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMessagePayload {
    pub message_id: Uuid,
    pub recipient_device: Uuid,
    pub ciphertext: Vec<u8>,
    pub is_pre_key: bool,
}

impl TryFrom<InboundMessagePayload> for NewMessagePayload {
    type Error = AppError;

    fn try_from(msg: InboundMessagePayload) -> Result<Self, Self::Error> {
        Ok(Self {
            message_id: msg.message_id,
            recipient_device: msg.recipient_device,
            ciphertext: BASE64_STANDARD_NO_PAD
                .decode(msg.ciphertext)
                .map_err(|e| AppError::InvalidB64(e.to_string()))?,
            is_pre_key: msg.is_pre_key,
        })
    }
}

#[derive(Clone, Deserialize)]
pub struct InboundMessagePayload {
    pub message_id: Uuid,
    pub recipient_device: Uuid,
    pub ciphertext: String,
    pub is_pre_key: bool,
}
