mod session;

use std::sync::Arc;

use diesel::{PgConnection, r2d2::ConnectionManager};
use r2d2::Pool;

use crate::services::{AuthService, KeyExchangeService, MessageRelayService};

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthService>,
    pub keys: Arc<dyn KeyExchangeService>,
    pub relay: Arc<dyn MessageRelayService>,
    pub oauth: crate::OAuthHandler,
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl AppState {
    #[must_use]
    pub fn new(
        auth: Arc<dyn AuthService>,
        keys: Arc<dyn KeyExchangeService>,
        relay: Arc<dyn MessageRelayService>,
        oauth: crate::OAuthHandler,
        pool: Pool<ConnectionManager<PgConnection>>,
    ) -> Self {
        Self {
            auth,
            keys,
            relay,
            oauth,
            pool,
        }
    }

    pub(crate) fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, crate::AppError> {
        self.pool
            .get()
            .map_err(|e| crate::AppError::PoolError(e.to_string()))
    }
}
