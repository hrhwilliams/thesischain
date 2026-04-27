use diesel::{Insertable, Queryable, Selectable};
use ed25519_dalek::Signature;
use serde::{Deserialize, Serialize};
use vodozemac::{Curve25519PublicKey, Ed25519PublicKey, Ed25519Signature};

use crate::{AppError, DeviceId, UserId, serialize_as_base64, serialize_as_base64_opt};

#[derive(Clone, Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::device)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Device {
    #[serde(rename(serialize = "device_id"))]
    pub id: DeviceId,
    pub user_id: UserId,
    #[serde(serialize_with = "serialize_as_base64_opt")]
    pub ed25519: Option<Vec<u8>>,
    #[serde(serialize_with = "serialize_as_base64_opt")]
    pub x25519: Option<Vec<u8>>,
}

#[derive(Debug, Serialize)]
pub struct HistoricalKey {
    pub device_id: DeviceId,
    pub chain_height: u64,
    #[serde(serialize_with = "serialize_as_base64")]
    pub x25519: Vec<u8>,
    #[serde(serialize_with = "serialize_as_base64")]
    pub ed25519: Vec<u8>,
    #[serde(serialize_with = "serialize_as_base64")]
    pub signature: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<InboundAuthorization>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::device)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDevice {
    pub user_id: UserId,
    pub ed25519: Option<Vec<u8>>,
    pub x25519: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct InboundDevice {
    pub device_id: Option<DeviceId>,
    pub ed25519: String,
    pub x25519: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<InboundAuthorization>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InboundAuthorization {
    pub authorizing_device_id: DeviceId,
    pub signature: String,
}

impl InboundAuthorization {
    pub fn verify(
        &self,
        previous_key: &Ed25519PublicKey,
        keys_signature: Signature,
    ) -> Result<(), AppError> {
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(previous_key.as_bytes())
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        let signature_bytes = Ed25519Signature::from_base64(&self.signature)
            .map_err(|_| AppError::InvalidSignature)?;
        let signature = Signature::from_bytes(&signature_bytes.to_bytes());

        verifying_key
            .verify_strict(&keys_signature.to_bytes(), &signature)
            .map_err(|e| AppError::ChallengeFailed(e.to_string()))
    }
}

impl NewDevice {
    pub fn from_network(user_id: UserId, device: &InboundDevice) -> Result<Self, AppError> {
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

        Ok(Self {
            user_id,
            ed25519: Some(ed25519.as_bytes().to_vec()),
            x25519: Some(x25519.as_bytes().to_vec()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};

    /*
       x25519: "gzpLJ9mEPG3hz5zBpz4jlcRlMkS6gW0p093Kl+5hJRo",
       ed25519: "lQVAT2liS82yLLrlSzO8LBfXkffRvhKcFMOi4zkw9JM",
       signature: "LPxZkj+a44bkqWJQAwfWatbKV6iQ4lqL6DEV6+B9kdPV1A1PNKC8QNliO24l+9fAwLRz9iDJQegrvWNnzMZZCw",

       x25519: "kU6sga8vcY25x2ML3Y31mjtjmZEAL9i8fDOtbb1c1xg",
       ed25519: "huJuHz7Y70Uay06AKlrAf5sLnNUk/XAv710TFH7Et3A",
       signature: "fGceS7RH1KRg9UgqCOzf/LeBKrgjuXZrO332r2Q4LJ10k63QJ6OR1UUIwgrHQel6wAA7urqvTWcBXcR1jAFkBw",
       authorization: "9VbUmEsAMU1mHcAMb4SBpOTJ9Zgk84f0+5M214SKiLbXrL6mBc3sOI6/aArVq5SzGPMK3hRiWi98JaHEYti6Ag",
    */

    const DEVICE_1_ED25519: &str = "lQVAT2liS82yLLrlSzO8LBfXkffRvhKcFMOi4zkw9JM";
    const DEVICE_2_SELF_SIG: &str =
        "fGceS7RH1KRg9UgqCOzf/LeBKrgjuXZrO332r2Q4LJ10k63QJ6OR1UUIwgrHQel6wAA7urqvTWcBXcR1jAFkBw";
    const DEVICE_2_BAD_SIG: &str =
        "fd20T/eTY7KQ9gpFHwSeJNyxrCZPlOm1RVx4NPPz8RTqkOFP91PIb2EK/GghvPKI/3VCLlLahAsv/AQJCGQMCw";
    const DEVICE_2_AUTHORIZATION: &str =
        "9VbUmEsAMU1mHcAMb4SBpOTJ9Zgk84f0+5M214SKiLbXrL6mBc3sOI6/aArVq5SzGPMK3hRiWi98JaHEYti6Ag";

    fn decode_signature(b64: &str) -> Signature {
        let bytes: [u8; 64] = BASE64_STANDARD_NO_PAD
            .decode(b64)
            .expect("valid base64")
            .try_into()
            .expect("64-byte ed25519 signature");
        Signature::from_bytes(&bytes)
    }

    #[test]
    fn authorization_verifies_known_rotation() {
        let prev_key = Ed25519PublicKey::from_base64(DEVICE_1_ED25519).expect("valid ed25519");
        let new_self_sig = decode_signature(DEVICE_2_SELF_SIG);

        let auth = InboundAuthorization {
            authorizing_device_id: DeviceId::new_v7(),
            signature: DEVICE_2_AUTHORIZATION.to_owned(),
        };

        auth.verify(&prev_key, new_self_sig)
            .expect("authorization from device 1 over device 2's self-sig must verify");
    }

    #[test]
    fn authorization_rejects_wrong_new_signature() {
        let prev_key = Ed25519PublicKey::from_base64(DEVICE_1_ED25519).expect("valid ed25519");

        let tampered = decode_signature(DEVICE_2_BAD_SIG);

        let auth = InboundAuthorization {
            authorizing_device_id: DeviceId::new_v7(),
            signature: DEVICE_2_AUTHORIZATION.to_owned(),
        };

        auth.verify(&prev_key, tampered)
            .expect_err("tampered self-sig must fail authorization check");
    }
}
