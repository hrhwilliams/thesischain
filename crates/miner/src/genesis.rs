use ed25519_dalek::SigningKey;
use uuid::Uuid;

use crate::crypto;
use crate::error::ChainError;
use crate::types::{Block, IdentityAttestation, Transaction};

/// Information needed to register an initial device in the genesis block.
pub struct GenesisDevice {
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub signing_key: SigningKey,
    pub x25519: [u8; 32],
}

/// Create a genesis block containing `RegisterDevice` transactions for the initial authority set.
///
/// Each device signs its own registration transaction. The bootstrap key signs the block itself.
/// The bootstrap key should also be registered as a device to become an authority.
///
/// If `backend_signing_key` is provided, each genesis device gets a proper identity attestation.
/// If `None`, a dummy attestation is used (only valid when `Chain::new` skips attestation checks).
pub fn create_genesis(
    bootstrap_key: &SigningKey,
    timestamp: u64,
    initial_devices: &[GenesisDevice],
    backend_signing_key: Option<&SigningKey>,
) -> Result<Block, ChainError> {
    let mut transactions = Vec::with_capacity(initial_devices.len());

    for (i, device) in initial_devices.iter().enumerate() {
        let ed25519 = device.signing_key.verifying_key().to_bytes();

        let attestation = if let Some(bk) = backend_signing_key {
            crypto::sign_attestation(device.user_id, device.device_id, timestamp, bk)?
        } else {
            // Dummy attestation — only valid when Chain has no backend_key set
            IdentityAttestation {
                user_id: device.user_id,
                device_id: device.device_id,
                issued_at: timestamp,
                backend_key: [0u8; 32],
                signature: ed25519_dalek::Signature::from_bytes(&[0u8; 64]),
            }
        };

        let tx = Transaction::RegisterDevice {
            user_id: device.user_id,
            device_id: device.device_id,
            ed25519,
            x25519: device.x25519,
            attestation,
        };

        #[allow(clippy::cast_possible_truncation)]
        let signed = crypto::sign_transaction(tx, i as u64, &device.signing_key)?;
        transactions.push(signed);
    }

    crypto::sign_block(0, timestamp, [0u8; 32], transactions, bootstrap_key)
}
