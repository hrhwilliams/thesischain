use ed25519_dalek::Signature;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A blockchain identity — the raw 32-byte ed25519 public key.
pub type ChainId = [u8; 32];

/// Header of a block in the chain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub index: u64,
    pub timestamp: u64,
    /// SHA-256 hash of the previous block (all zeros for genesis).
    pub previous_hash: [u8; 32],
    /// SHA-256 hash of the bincode-serialized transactions vector.
    pub transactions_hash: [u8; 32],
    /// Ed25519 public key of the block author.
    pub author: ChainId,
    /// Ed25519 signature over `(index || timestamp || previous_hash || transactions_hash)`.
    pub signature: Signature,
}

/// A block in the chain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<SignedTransaction>,
}

/// A transaction with its cryptographic proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub payload: Transaction,
    /// Ed25519 public key of the transaction signer.
    pub signer: ChainId,
    /// Ed25519 signature over the bincode-serialized `(payload, nonce)`.
    pub signature: Signature,
    /// Monotonically increasing nonce for replay protection.
    pub nonce: u64,
}

/// Backend-signed identity attestation for dual-authority key registration.
///
/// Required for `RegisterDevice` transactions to prevent either the backend
/// or the chain from independently distributing malicious keys.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityAttestation {
    pub user_id: Uuid,
    pub device_id: Uuid,
    /// Unix timestamp (seconds) when the attestation was issued.
    pub issued_at: u64,
    /// Ed25519 public key of the backend that signed this attestation.
    pub backend_key: [u8; 32],
    /// Ed25519 signature over `bincode(user_id, device_id, issued_at)`.
    pub signature: Signature,
}

/// Operations that can be recorded on the chain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Transaction {
    /// Register a new device with its identity keys.
    /// The signer's ed25519 key must match the `ed25519` field.
    /// Requires a backend-signed identity attestation.
    RegisterDevice {
        user_id: Uuid,
        device_id: Uuid,
        ed25519: [u8; 32],
        x25519: [u8; 32],
        attestation: IdentityAttestation,
    },
    /// Update keys for an existing device.
    /// Must be signed by the device's current ed25519 key.
    UpdateDeviceKeys {
        device_id: Uuid,
        new_ed25519: [u8; 32],
        new_x25519: [u8; 32],
    },
    /// Revoke a device, removing it from the active key directory.
    /// Must be signed by the device's current ed25519 key.
    RevokeDevice { device_id: Uuid },
}
