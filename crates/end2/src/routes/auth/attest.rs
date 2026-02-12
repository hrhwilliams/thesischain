use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ApiError, AppError, AppState, User};

#[derive(Deserialize)]
pub struct AttestRequest {
    pub device_id: Uuid,
}

#[derive(Serialize)]
pub struct AttestResponse {
    pub attestation: miner::IdentityAttestation,
}

pub async fn attest(
    user: User,
    State(state): State<AppState>,
    Json(req): Json<AttestRequest>,
) -> Result<Json<AttestResponse>, ApiError> {
    let key = state
        .attestation_key
        .as_ref()
        .ok_or_else(|| AppError::UserError("attestation not configured".into()))?;

    let issued_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_secs();

    let attestation = miner::sign_attestation(user.id, req.device_id, issued_at, key)
        .map_err(|e| AppError::UserError(e.to_string()))?;

    Ok(Json(AttestResponse { attestation }))
}
