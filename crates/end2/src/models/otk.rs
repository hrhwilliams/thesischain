use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::one_time_key)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Otk {
    pub id: Uuid,
    pub device_id: Uuid,
    pub otk: Vec<u8>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::one_time_key)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewOtk {
    pub device_id: Uuid,
    pub otk: [u8; 32],
}

#[derive(Serialize)]
pub struct OutboundOtk {
    pub otk: String,
}

impl From<Otk> for OutboundOtk {
    fn from(value: Otk) -> Self {
        Self {
            otk: BASE64_STANDARD_NO_PAD.encode(value.otk),
        }
    }
}

#[derive(Deserialize)]
pub struct InboundOtks {
    pub otks: Vec<String>,
    pub signature: String,
}
