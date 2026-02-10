use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::schema::web_session;
use crate::{AppError, WebSession};

use super::AppState;

impl AppState {
    #[tracing::instrument(skip(self))]
    pub async fn new_session(&self) -> Result<WebSession, AppError> {
        let mut conn = self.get_conn()?;

        let session = tokio::task::spawn_blocking(move || {
            diesel::insert_into(web_session::table)
                .default_values()
                .returning(WebSession::as_returning())
                .get_result(&mut conn)
        })
        .await??;

        Ok(session)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_session(&self, session_id: Uuid) -> Result<Option<WebSession>, AppError> {
        let mut conn = self.get_conn()?;

        let session = tokio::task::spawn_blocking(move || {
            web_session::table
                .find(session_id)
                .select(WebSession::as_select())
                .first(&mut conn)
                .optional()
        })
        .await??;

        Ok(session)
    }

    pub async fn insert_into_session<T: Serialize>(
        &self,
        web_session: WebSession,
        key: String,
        value: T,
    ) -> Result<WebSession, AppError> {
        let mut conn = self.get_conn()?;

        let blob = match web_session.blob {
            Value::Object(mut m) => {
                m.insert(
                    key,
                    serde_json::to_value(value).map_err(|e| AppError::ValueError(e.to_string()))?,
                );
                Value::Object(m)
            }
            _ => unreachable!("only blob should be stored in web_session table"),
        };

        let web_session = tokio::task::spawn_blocking(move || {
            diesel::update(web_session::table)
                .filter(web_session::id.eq(web_session.id))
                .set(web_session::blob.eq(blob))
                .get_result(&mut conn)
        })
        .await??;

        Ok(web_session)
    }

    pub async fn get_from_session<T: DeserializeOwned>(
        &self,
        web_session: &WebSession,
        key: &str,
    ) -> Result<Option<T>, AppError> {
        let mut conn = self.get_conn()?;

        let web_session_id = web_session.id;

        let blob = tokio::task::spawn_blocking(move || {
            web_session::table
                .find(web_session_id)
                .select(web_session::blob)
                .get_result(&mut conn)
        })
        .await??;

        let value = match blob {
            Value::Object(m) => m
                .get(key)
                .cloned()
                .and_then(|v| serde_json::from_value(v).ok()),
            _ => unreachable!("only blob should be stored in web_session table"),
        };

        Ok(value)
    }

    pub async fn remove_from_session<T: DeserializeOwned>(
        &self,
        web_session: WebSession,
        key: &str,
    ) -> Result<Option<(T, WebSession)>, AppError> {
        let mut conn = self.get_conn()?;

        let (value, blob) = match web_session.blob {
            Value::Object(mut m) => {
                let value = m.remove(key).and_then(|v| serde_json::from_value(v).ok());
                (value, Value::Object(m))
            }
            _ => unreachable!("only blob should be stored in web_session table"),
        };

        if let Some(value) = value {
            let web_session = tokio::task::spawn_blocking(move || {
                diesel::update(web_session::table)
                    .filter(web_session::id.eq(web_session.id))
                    .set(web_session::blob.eq(blob))
                    .get_result(&mut conn)
            })
            .await??;

            Ok(Some((value, web_session)))
        } else {
            Ok(None)
        }
    }
}
