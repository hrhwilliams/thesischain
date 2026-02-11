use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::util::serialize_as_base64;

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::one_time_key)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Otk {
    pub id: Uuid,
    pub device_id: Uuid,
    #[serde(serialize_with = "serialize_as_base64")]
    pub otk: Vec<u8>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::one_time_key)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewOtk {
    pub device_id: Uuid,
    pub otk: [u8; 32],
}

#[derive(Deserialize)]
pub struct InboundOtks {
    pub created: Vec<String>,
    pub removed: Vec<String>,
    pub created_signature: String,
    pub removed_signature: Option<String>,
}
