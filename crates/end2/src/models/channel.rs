use crate::{ChannelId, Device, User, UserId};
use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;

#[derive(Clone, Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::channel)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Channel {
    #[serde(rename(serialize = "channel_id"))]
    pub id: ChannelId,
}

#[derive(Clone, Debug, Insertable, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::channel_participant)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ChannelParticipant {
    pub channel_id: ChannelId,
    pub user_id: UserId,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChannelInfo {
    pub channel_id: ChannelId,
    pub participants: Vec<User>,
    pub devices: Vec<Device>,
}
