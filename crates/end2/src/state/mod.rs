use std::sync::Arc;

use diesel::{PgConnection, r2d2::ConnectionManager};
use ed25519_dalek::SigningKey;
use r2d2::Pool;
use tokio::sync::{RwLock, broadcast};

use crate::{
    WebSessionService,
    services::{AuthService, DeviceKeyService, MessageRelayService, OtkService},
};

#[derive(Clone)]
pub enum AppEvent {}

#[derive(Clone)]
pub struct AppState<
    A: AuthService,
    D: DeviceKeyService,
    O: OtkService,
    R: MessageRelayService,
    W: WebSessionService,
> {
    pub auth: A,
    pub device_keys: D,
    pub otks: O,
    pub relay: R,
    pub web_sessions: W,
    pub signing_key: Arc<SigningKey>,
    pub miners: Arc<RwLock<Vec<miner::MinerInfo>>>,
    pool: Pool<ConnectionManager<PgConnection>>,
    /// Sends AppEvents to subscribers
    broadcaster: broadcast::Sender<AppEvent>,
}

impl<
    A: AuthService,
    D: DeviceKeyService,
    O: OtkService,
    R: MessageRelayService,
    W: WebSessionService,
> AppState<A, D, O, R, W>
{
    #[must_use]
    pub fn new(
        auth: A,
        device_keys: D,
        otks: O,
        relay: R,
        web_sessions: W,
        pool: Pool<ConnectionManager<PgConnection>>,
        signing_key: SigningKey,
    ) -> Self {
        let (broadcaster, _) = broadcast::channel(256);

        Self {
            auth,
            device_keys,
            otks,
            relay,
            web_sessions,
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
}
