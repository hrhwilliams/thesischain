mod channel;
mod device;
mod message;
mod session;
mod user;

use std::collections::HashMap;
use std::sync::Arc;

use diesel::{PgConnection, r2d2::ConnectionManager};
use r2d2::Pool;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;

use crate::models::User;
use crate::WsEvent;

#[derive(Clone)]
pub struct AppState {
    pub oauth: crate::OAuthHandler,
    pool: Pool<ConnectionManager<PgConnection>>,
    user_websockets: Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsEvent>>>>,
    device_websockets: Arc<RwLock<HashMap<Uuid, mpsc::Sender<WsEvent>>>>,
}

impl AppState {
    #[must_use]
    pub fn new(oauth: crate::OAuthHandler, pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self {
            oauth,
            pool,
            user_websockets: Arc::default(),
            device_websockets: Arc::default(),
        }
    }

    pub async fn register_device(&self, device_id: Uuid, device_tx: mpsc::Sender<WsEvent>) {
        let mut device_websockets = self.device_websockets.write().await;
        device_websockets.insert(device_id, device_tx);
    }

    pub async fn unregister_device(&self, device_id: Uuid) {
        let mut device_websockets = self.device_websockets.write().await;
        device_websockets.remove(&device_id);
    }

    pub async fn get_broadcaster(&self, user: &User) -> broadcast::Sender<WsEvent> {
        let mut user_websockets = self.user_websockets.write().await;
        let sender = user_websockets
            .entry(user.id)
            .or_insert_with(|| broadcast::Sender::new(128));

        sender.clone()
    }

    pub async fn get_broadcaster_for_device(
        &self,
        device_id: Uuid,
    ) -> Option<mpsc::Sender<WsEvent>> {
        let device_websockets = self.device_websockets.read().await;
        device_websockets.get(&device_id).cloned()
    }

    #[tracing::instrument(skip(self))]
    pub async fn notify_user(&self, user: &User, event: WsEvent) {
        tracing::info!("sending event");
        let broadcaster = self.get_broadcaster(user).await;
        match broadcaster.send(event) {
            Ok(_) => {}
            Err(e) => tracing::error!("failed to notify user: {}", e),
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
