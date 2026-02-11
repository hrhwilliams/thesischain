use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;
use uuid::Uuid;

use crate::{Device, User};

#[derive(Clone, Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Channel {
    #[serde(rename(serialize = "channel_id"))]
    pub id: Uuid,
}

#[derive(Clone, Debug, Insertable, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::channel_participant)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChannelParticipant {
    pub channel_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChannelInfo {
    pub channel_id: Uuid,
    pub participants: Vec<User>,
    pub devices: Vec<Device>,
}
