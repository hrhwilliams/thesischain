use crate::{ApiError, AppError, AppState, DeviceId, User};
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AttestRequest {
    pub device_id: DeviceId,
}

#[derive(Serialize)]
pub struct AttestResponse {
    pub attestation: miner::IdentityAttestation,
}

pub async fn attest(
    user: User,
    State(state): State<AppState>,
    Json(AttestRequest { device_id }): Json<AttestRequest>,
) -> Result<Json<AttestResponse>, ApiError> {
    let key = state.signing_key.as_ref();

    let issued_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_secs();

    let attestation =
        miner::sign_attestation(user.id.into_inner(), device_id.into_inner(), issued_at, key)
            .map_err(|e| AppError::UserError(e.to_string()))?;

    Ok(Json(AttestResponse { attestation }))
}
