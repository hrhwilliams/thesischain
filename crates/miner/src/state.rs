use std::collections::{HashMap, HashSet};

use ed25519_dalek::VerifyingKey;
use uuid::Uuid;

use crate::crypto;
use crate::error::ChainError;
use crate::types::{ChainId, SignedTransaction, Transaction};

/// A device's record in the key directory.
#[derive(Clone, Debug, serde::Serialize)]
pub struct DeviceRecord {
    pub device_id: Uuid,
    pub user_id: Uuid,
    pub ed25519: [u8; 32],
    pub x25519: [u8; 32],
    pub registered_at_block: u64,
    pub revoked: bool,
}

/// The derived world state — a key directory built by replaying all chain transactions.
#[derive(Clone, Debug, Default)]
pub struct KeyDirectory {
    /// `device_id` -> device record
    devices: HashMap<Uuid, DeviceRecord>,
    /// `user_id` -> list of device IDs
    user_devices: HashMap<Uuid, Vec<Uuid>>,
    /// `ed25519 public key` -> device ID (reverse index)
    key_to_device: HashMap<ChainId, Uuid>,
    /// Per-signer nonce counter (must be strictly increasing).
    nonces: HashMap<ChainId, u64>,
    /// Set of ed25519 keys that are authorities (all non-revoked registered devices).
    authorities: HashSet<ChainId>,
}

impl KeyDirectory {
    /// Look up a device by its UUID.
    #[must_use]
    pub fn get_device(&self, device_id: Uuid) -> Option<&DeviceRecord> {
        self.devices.get(&device_id)
    }

    /// Get all devices belonging to a user.
    #[must_use]
    pub fn get_user_devices(&self, user_id: Uuid) -> Vec<&DeviceRecord> {
        self.user_devices
            .get(&user_id)
            .map(|ids| ids.iter().filter_map(|id| self.devices.get(id)).collect())
            .unwrap_or_default()
    }

    /// Check whether a given ed25519 key is an authority.
    #[must_use]
    pub fn is_authority(&self, key: &ChainId) -> bool {
        self.authorities.contains(key)
    }

    /// Look up which device a chain identity (ed25519 key) belongs to.
    #[must_use]
    pub fn device_for_key(&self, key: &ChainId) -> Option<Uuid> {
        self.key_to_device.get(key).copied()
    }

    /// Verify that a transaction's nonce is valid (strictly greater than the last seen nonce).
    pub fn verify_nonce(&self, signer: &ChainId, nonce: u64) -> Result<(), ChainError> {
        let expected = self.nonces.get(signer).map_or(0, |n| n + 1);
        if nonce < expected {
            return Err(ChainError::InvalidNonce {
                expected,
                got: nonce,
            });
        }
        Ok(())
    }

    /// Apply a signed transaction to the state, updating the key directory.
    /// Assumes the transaction signature has already been verified.
    ///
    /// When `backend_key` is `Some`, `RegisterDevice` transactions must include
    /// a valid identity attestation signed by the backend.
    pub fn apply_transaction(
        &mut self,
        tx: &SignedTransaction,
        block_index: u64,
        backend_key: Option<&VerifyingKey>,
    ) -> Result<(), ChainError> {
        self.verify_nonce(&tx.signer, tx.nonce)?;
        self.nonces.insert(tx.signer, tx.nonce);

        match &tx.payload {
            Transaction::RegisterDevice {
                user_id,
                device_id,
                ed25519,
                x25519,
                attestation,
            } => {
                if self.devices.contains_key(device_id) {
                    return Err(ChainError::DuplicateDeviceId(*device_id));
                }

                // The signer must be the device's own ed25519 key
                if tx.signer != *ed25519 {
                    return Err(ChainError::UnauthorizedSigner);
                }

                // Verify backend identity attestation when configured
                if let Some(bk) = backend_key {
                    crypto::verify_attestation(attestation, bk)?;
                    if attestation.user_id != *user_id || attestation.device_id != *device_id {
                        return Err(ChainError::InvalidAttestation(
                            "attestation fields do not match transaction".into(),
                        ));
                    }
                }

                let record = DeviceRecord {
                    device_id: *device_id,
                    user_id: *user_id,
                    ed25519: *ed25519,
                    x25519: *x25519,
                    registered_at_block: block_index,
                    revoked: false,
                };

                self.devices.insert(*device_id, record);
                self.user_devices
                    .entry(*user_id)
                    .or_default()
                    .push(*device_id);
                self.key_to_device.insert(*ed25519, *device_id);
                self.authorities.insert(*ed25519);

                Ok(())
            }

            Transaction::UpdateDeviceKeys {
                device_id,
                new_ed25519,
                new_x25519,
            } => {
                let record = self
                    .devices
                    .get_mut(device_id)
                    .ok_or(ChainError::UnknownDevice(*device_id))?;

                if record.revoked {
                    return Err(ChainError::UnknownDevice(*device_id));
                }

                // Must be signed by the current device key
                if tx.signer != record.ed25519 {
                    return Err(ChainError::UnauthorizedSigner);
                }

                // Update reverse index
                self.key_to_device.remove(&record.ed25519);
                self.authorities.remove(&record.ed25519);

                record.ed25519 = *new_ed25519;
                record.x25519 = *new_x25519;

                self.key_to_device.insert(*new_ed25519, *device_id);
                self.authorities.insert(*new_ed25519);

                Ok(())
            }

            Transaction::RevokeDevice { device_id } => {
                let record = self
                    .devices
                    .get_mut(device_id)
                    .ok_or(ChainError::UnknownDevice(*device_id))?;

                if record.revoked {
                    return Err(ChainError::UnknownDevice(*device_id));
                }

                // Must be signed by the device's own key
                if tx.signer != record.ed25519 {
                    return Err(ChainError::UnauthorizedSigner);
                }

                record.revoked = true;
                self.authorities.remove(&record.ed25519);

                Ok(())
            }
        }
    }
}
