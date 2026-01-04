use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppError;

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub ed25519: Vec<u8>,
    pub curve25519: Vec<u8>,
}

#[derive(Debug, Insertable, Serialize)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUser {
    pub username: String,
    pub ed25519: Vec<u8>,
    pub curve25519: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct NewUserB64 {
    pub username: String,
    pub ed25519: String,
    pub curve25519: String,
    pub signature: String,
}

impl TryFrom<NewUserB64> for NewUser {
    type Error = AppError;

    fn try_from(value: NewUserB64) -> Result<Self, Self::Error> {
        let ed25519 = BASE64_STANDARD_NO_PAD.decode(value.ed25519)?;
        let curve25519 = BASE64_STANDARD_NO_PAD.decode(value.curve25519)?;
        let signature = BASE64_STANDARD_NO_PAD.decode(value.signature)?;

        Ok(Self {
            username: value.username,
            ed25519,
            curve25519,
            signature,
        })
    }
}

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::challenge)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Challenge {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::one_time_key)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Otk {
    pub id: Uuid,
    pub user_id: Uuid,
    pub otk: Vec<u8>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::one_time_key)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewOtk {
    pub user_id: Uuid,
    pub otk: [u8; 32],
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::challenge)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewChallenge {
    pub user_id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatMessage {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author: Uuid,
    pub content: Vec<u8>,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::message)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewChatMessage {
    pub channel_id: Uuid,
    pub author: Uuid,
    pub content: Vec<u8>,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::session)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::session)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSession {
    pub user_id: Uuid,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Channel {
    pub id: Uuid,
    pub sender: Uuid,
    pub receiver: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewChannel {
    pub sender: Uuid,
    pub receiver: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ChannelResponse {
    pub id: Uuid,
    pub sender: String,
    pub receiver: String,
}

#[derive(Serialize)]
pub struct KeyResponse {
    pub kind: String,
    pub key: String,
}
