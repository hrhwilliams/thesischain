use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::AppError;

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::message_payload)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MessagePayload {
    pub message_id: Uuid,
    pub recipient_device_id: Uuid,
    pub ciphertext: Vec<u8>,
    pub is_pre_key: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::message_payload)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMessagePayload {
    pub message_id: Uuid,
    pub recipient_device_id: Uuid,
    pub ciphertext: Vec<u8>,
    pub is_pre_key: bool,
}

#[derive(Clone, Deserialize)]
pub struct InboundMessagePayload {
    pub recipient_device_id: Uuid,
    pub ciphertext: String,
    pub is_pre_key: bool,
}

impl InboundMessagePayload {
    pub fn to_new_message(self, message_id: Uuid) -> Result<NewMessagePayload, AppError> {
        Ok(NewMessagePayload {
            message_id,
            recipient_device_id: self.recipient_device_id,
            ciphertext: BASE64_STANDARD_NO_PAD.decode(self.ciphertext)?,
            is_pre_key: self.is_pre_key,
        })
    }
}
