use base64::{
    Engine,
    prelude::{BASE64_STANDARD, BASE64_STANDARD_NO_PAD},
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppError;

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub ed25519: Vec<u8>,
    pub curve25519: Vec<u8>,
}

#[derive(Debug, Insertable, Serialize)]
#[diesel(table_name = crate::schema::users)]
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

#[derive(Insertable)]
#[diesel(table_name = crate::schema::challenge)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewChallenge {
    pub user_id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChatMessage {
    pub id: Uuid,
    pub room_id: Uuid,
    pub author: Uuid,
    pub content: String,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewChatMessage {
    pub room_id: Uuid,
    pub author: Uuid,
    pub content: String,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewSession {
    pub user_id: Uuid,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::rooms)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Room {
    pub id: Uuid,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::room_participants)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RoomParticipant {
    pub room_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::room_participants)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewRoomParticipant {
    pub room_id: Uuid,
    pub user_id: Uuid,
}
