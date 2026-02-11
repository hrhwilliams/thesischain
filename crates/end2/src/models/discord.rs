use std::num::ParseIntError;

use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::RegistrationError;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::discord_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DiscordInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub discord_id: i64,
    pub discord_username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::discord_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDiscordInfo {
    pub user_id: Uuid,
    pub discord_id: i64,
    pub discord_username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InboundDiscordInfo {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
}

impl NewDiscordInfo {
    pub fn from_inbound(
        inbound: InboundDiscordInfo,
        user_id: Uuid,
    ) -> Result<Self, RegistrationError> {
        Ok(Self {
            user_id,
            discord_id: inbound
                .id
                .parse()
                .map_err(|e: ParseIntError| RegistrationError::InvalidDiscordId(e.to_string()))?,
            discord_username: inbound.username,
            global_name: inbound.global_name,
            avatar: inbound.avatar,
        })
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::discord_auth_token)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DiscordAuthToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: Vec<u8>,
    pub refresh_token: Option<Vec<u8>>,
    pub expires: Option<OffsetDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::discord_auth_token)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDiscordAuthToken {
    pub user_id: Uuid,
    pub access_token: Vec<u8>,
    pub refresh_token: Option<Vec<u8>>,
    pub expires: Option<OffsetDateTime>,
}

#[derive(Serialize)]
pub struct InboundDiscordAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires: Option<u64>,
}
