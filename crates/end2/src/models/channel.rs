use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::OutboundDevice;

#[derive(Clone, Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Channel {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewChannel {
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
}

#[derive(Serialize)]
pub struct ChannelResponse {
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
}

#[derive(Serialize)]
pub struct ChannelInfo {
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub nickname: Option<String>,
    pub devices: Vec<OutboundDevice>,
}
