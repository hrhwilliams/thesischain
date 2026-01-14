use diesel::prelude::*;
use ed25519_dalek::Signature;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vodozemac::{Curve25519PublicKey, Ed25519PublicKey, Ed25519Signature};

use crate::{AppError, serialize_as_base64_opt};

#[derive(Clone, Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::device)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Device {
    #[serde(rename(serialize = "device_id"))]
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(serialize_with = "serialize_as_base64_opt")]
    pub ed25519: Option<Vec<u8>>,
    #[serde(serialize_with = "serialize_as_base64_opt")]
    pub x25519: Option<Vec<u8>>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::device)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDevice {
    pub user_id: Uuid,
    pub ed25519: Option<Vec<u8>>,
    pub x25519: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct InboundDevice {
    pub device_id: Option<Uuid>,
    pub ed25519: String,
    pub x25519: String,
    pub signature: String,
}

impl NewDevice {
    pub fn from_network(user_id: Uuid, device: InboundDevice) -> Result<Self, AppError> {
        let x25519 = Curve25519PublicKey::from_base64(&device.x25519)
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;
        let ed25519 = Ed25519PublicKey::from_base64(&device.ed25519)
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;
        let signature = Ed25519Signature::from_base64(&device.signature)
            .map_err(|_| AppError::InvalidSignature)?;

        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(ed25519.as_bytes())
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        let signature = Signature::from_bytes(&signature.to_bytes());

        let message = [x25519.as_bytes() as &[u8], ed25519.as_bytes()].concat();

        verifying_key
            .verify_strict(&message, &signature)
            .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

        Ok(NewDevice {
            user_id,
            ed25519: Some(ed25519.as_bytes().to_vec()),
            x25519: Some(x25519.as_bytes().to_vec()),
        })
    }
}
