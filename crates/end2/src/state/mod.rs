mod session;

use std::collections::HashMap;
use std::sync::Arc;

use diesel::{PgConnection, r2d2::ConnectionManager};
use ed25519_dalek::SigningKey;
use r2d2::Pool;
use tokio::sync::{RwLock, broadcast};

use crate::{
    OAuthHandler,
    services::{AuthService, DeviceKeyService, MessageRelayService, OtkService},
};

#[derive(Clone)]
pub enum AppEvent {}

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<dyn AuthService>,
    pub device_keys: Arc<dyn DeviceKeyService>,
    pub otks: Arc<dyn OtkService>,
    pub relay: Arc<dyn MessageRelayService>,
    pub oauth: HashMap<String, OAuthHandler>,
    pub signing_key: Arc<SigningKey>,
    pub miners: Arc<RwLock<Vec<miner::MinerInfo>>>,
    pool: Pool<ConnectionManager<PgConnection>>,
    /// Sends AppEvents to subscribers
    broadcaster: broadcast::Sender<AppEvent>,
}

impl AppState {
    #[must_use]
    pub fn new(
        auth: Arc<dyn AuthService>,
        device_keys: Arc<dyn DeviceKeyService>,
        otks: Arc<dyn OtkService>,
        relay: Arc<dyn MessageRelayService>,
        oauth: HashMap<String, OAuthHandler>,
        pool: Pool<ConnectionManager<PgConnection>>,
        signing_key: SigningKey,
    ) -> Self {
        let (broadcaster, _) = broadcast::channel(256);

        Self {
            auth,
            device_keys,
            otks,
            relay,
            oauth,
            signing_key: Arc::new(signing_key),
            miners: Arc::new(RwLock::new(Vec::new())),
            pool,
            broadcaster,
        }
    }

    pub(crate) fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, crate::AppError> {
        self.pool
            .get()
            .map_err(|e| crate::AppError::PoolError(e.to_string()))
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.broadcaster.subscribe()
    }

    pub fn get_oauth_handler(&self, handler_name: &str) -> Option<&OAuthHandler> {
        self.oauth.get(handler_name)
    }
}
