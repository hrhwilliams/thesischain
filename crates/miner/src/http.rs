use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router, routing};
use ed25519_dalek::{SigningKey, VerifyingKey};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, RwLock, mpsc};
use uuid::Uuid;

use crate::chain::Chain;
use crate::crypto;
use crate::types::{SignedTransaction, Transaction};

/// How submitted transactions are routed.
#[derive(Clone)]
enum TxSink {
    /// Standalone mode: txs accumulate in a local mempool; `/mine` endpoint
    /// triggers block production.
    Local {
        pending_txs: Arc<Mutex<Vec<SignedTransaction>>>,
        block_author_key: Arc<SigningKey>,
    },
    /// Integrated mode: txs are forwarded to a running P2P [`Node`] via channel.
    /// Block production is handled by the node's timer.
    Channel(mpsc::Sender<SignedTransaction>),
}

/// Shared state for the miner HTTP API.
#[derive(Clone)]
pub struct MinerState {
    chain: Arc<RwLock<Chain>>,
    tx_sink: TxSink,
    backend_verifying_key: Option<VerifyingKey>,
}

/// HTTP API for interacting with the blockchain.
pub struct MinerApi {
    router: Router,
    chain: Arc<RwLock<Chain>>,
}

impl MinerApi {
    /// Standalone mode: manages its own mempool and block production via `/mine`.
    #[must_use]
    pub fn new(
        chain: Arc<RwLock<Chain>>,
        block_author_key: SigningKey,
        backend_verifying_key: Option<VerifyingKey>,
    ) -> Self {
        let state = MinerState {
            chain: Arc::clone(&chain),
            tx_sink: TxSink::Local {
                pending_txs: Arc::new(Mutex::new(Vec::new())),
                block_author_key: Arc::new(block_author_key),
            },
            backend_verifying_key,
        };

        let router = Router::new()
            .route("/tx", routing::post(submit_tx))
            .route("/mine", routing::post(mine_block))
            .route("/height", routing::get(get_height))
            .route("/device/{device_id}", routing::get(get_device))
            .route("/user/{user_id}/devices", routing::get(get_user_devices))
            .with_state(state);

        Self { router, chain }
    }

    /// Integrated mode: forwards transactions to a running P2P node via channel.
    /// Block production is handled by the node, so `/mine` is not available.
    #[must_use]
    pub fn integrated(
        chain: Arc<RwLock<Chain>>,
        tx_sender: mpsc::Sender<SignedTransaction>,
        backend_verifying_key: Option<VerifyingKey>,
    ) -> Self {
        let state = MinerState {
            chain: Arc::clone(&chain),
            tx_sink: TxSink::Channel(tx_sender),
            backend_verifying_key,
        };

        let router = Router::new()
            .route("/tx", routing::post(submit_tx))
            .route("/height", routing::get(get_height))
            .route("/device/{device_id}", routing::get(get_device))
            .route("/user/{user_id}/devices", routing::get(get_user_devices))
            .with_state(state);

        Self { router, chain }
    }

    /// Returns the shared chain reference (for the backend to use the same chain).
    #[must_use]
    pub const fn chain(&self) -> &Arc<RwLock<Chain>> {
        &self.chain
    }

    /// Run the HTTP server.
    ///
    /// # Errors
    /// Returns an error if the server fails to start.
    pub async fn run(self, listener: TcpListener) -> Result<(), std::io::Error> {
        axum::serve(listener, self.router).await
    }
}

/// Submit a signed transaction to the mempool (or forward to node).
async fn submit_tx(
    State(state): State<MinerState>,
    Json(tx): Json<SignedTransaction>,
) -> impl IntoResponse {
    // Verify device ed25519 signature
    if let Err(e) = crypto::verify_transaction(&tx) {
        return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
    }

    // Verify backend attestation for RegisterDevice transactions
    if let Transaction::RegisterDevice {
        ref attestation, ..
    } = tx.payload
        && let Some(ref bk) = state.backend_verifying_key
        && let Err(e) = crypto::verify_attestation(attestation, bk)
    {
        return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
    }

    match &state.tx_sink {
        TxSink::Local { pending_txs, .. } => {
            pending_txs.lock().await.push(tx);
            StatusCode::ACCEPTED.into_response()
        }
        TxSink::Channel(sender) => {
            if sender.send(tx).await.is_err() {
                return (StatusCode::INTERNAL_SERVER_ERROR, "node channel closed")
                    .into_response();
            }
            StatusCode::ACCEPTED.into_response()
        }
    }
}

/// Force block production from pending transactions (standalone mode only).
#[allow(clippy::significant_drop_tightening)]
async fn mine_block(State(state): State<MinerState>) -> impl IntoResponse {
    let (pending_txs, block_author_key) = match &state.tx_sink {
        TxSink::Local {
            pending_txs,
            block_author_key,
        } => (Arc::clone(pending_txs), Arc::clone(block_author_key)),
        TxSink::Channel(_) => {
            return (StatusCode::NOT_FOUND, "mining not available in integrated mode")
                .into_response();
        }
    };

    let txs: Vec<SignedTransaction> = {
        let mut pending = pending_txs.lock().await;
        std::mem::take(&mut *pending)
    };

    if txs.is_empty() {
        return StatusCode::NO_CONTENT.into_response();
    }

    let mut chain = state.chain.write().await;

    let previous_hash = match chain.head_hash() {
        Ok(h) => h,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let index = chain.height();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_secs();

    let block = match crypto::sign_block(index, timestamp, previous_hash, txs, &block_author_key) {
        Ok(b) => b,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match chain.append(block) {
        Ok(()) => Json(serde_json::json!({ "height": chain.height() })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Current chain height.
async fn get_height(State(state): State<MinerState>) -> impl IntoResponse {
    let chain = state.chain.read().await;
    Json(serde_json::json!({ "height": chain.height() }))
}

/// Look up a device by ID from chain state.
#[allow(clippy::significant_drop_tightening)]
async fn get_device(
    State(state): State<MinerState>,
    Path(device_id): Path<Uuid>,
) -> impl IntoResponse {
    let chain = state.chain.read().await;
    chain.state().get_device(device_id).map_or_else(
        || StatusCode::NOT_FOUND.into_response(),
        |record| Json(serde_json::to_value(record).expect("serializable")).into_response(),
    )
}

/// Get all devices for a user from chain state.
#[allow(clippy::significant_drop_tightening)]
async fn get_user_devices(
    State(state): State<MinerState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let devices: Vec<_> = state
        .chain
        .read()
        .await
        .state()
        .get_user_devices(user_id)
        .into_iter()
        .cloned()
        .collect();
    Json(devices)
}
