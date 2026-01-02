use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub pass: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub pass: &'a str,
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

#[derive(Queryable, Selectable)]
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
#[diesel(table_name = crate::schema::message_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MessageRequest {
    pub id: Uuid,
    pub sender: Uuid,
    pub receiver: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::message_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMessageRequest {
    pub sender: Uuid,
    pub receiver: Uuid,
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
