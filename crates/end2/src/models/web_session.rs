use crate::SessionId;
use diesel::{Insertable, Queryable, Selectable};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Insertable, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::web_session)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WebSession {
    pub id: SessionId,
    pub blob: Value,
}
