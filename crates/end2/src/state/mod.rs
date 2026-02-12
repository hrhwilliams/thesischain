mod session;

use std::sync::Arc;

use diesel::{PgConnection, r2d2::ConnectionManager};
use r2d2::Pool;

use crate::services::{AuthService, DeviceKeyService, MessageRelayService, OtkService};

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthService>,
    pub device_keys: Arc<dyn DeviceKeyService>,
    pub otks: Arc<dyn OtkService>,
    pub relay: Arc<dyn MessageRelayService>,
    pub oauth: crate::OAuthHandler,
    /// Ed25519 signing key for producing identity attestations.
    /// When set, enables the `/auth/attest` endpoint.
    pub attestation_key: Option<Arc<ed25519_dalek::SigningKey>>,
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl AppState {
    #[must_use]
    pub fn new(
        auth: Arc<dyn AuthService>,
        device_keys: Arc<dyn DeviceKeyService>,
        otks: Arc<dyn OtkService>,
        relay: Arc<dyn MessageRelayService>,
        oauth: crate::OAuthHandler,
        pool: Pool<ConnectionManager<PgConnection>>,
    ) -> Self {
        Self {
            auth,
            device_keys,
            otks,
            relay,
            oauth,
            attestation_key: None,
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
