use std::fmt;
use uuid::Uuid;

/// Errors that can occur during chain operations.
#[derive(Debug)]
pub enum ChainError {
    InvalidBlockIndex { expected: u64, got: u64 },
    InvalidPreviousHash,
    InvalidTimestamp,
    InvalidTransactionsHash,
    InvalidBlockSignature,
    UnauthorizedBlockAuthor,
    InvalidTransactionSignature,
    DuplicateDeviceId(Uuid),
    UnknownDevice(Uuid),
    UnauthorizedSigner,
    InvalidNonce { expected: u64, got: u64 },
    SerializationError(String),
    InvalidKey(String),
    InvalidAttestation(String),
}

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBlockIndex { expected, got } => {
                write!(f, "invalid block index: expected {expected}, got {got}")
            }
            Self::InvalidPreviousHash => write!(f, "previous hash does not match"),
            Self::InvalidTimestamp => write!(f, "block timestamp is before previous block"),
            Self::InvalidTransactionsHash => write!(f, "transactions hash does not match"),
            Self::InvalidBlockSignature => write!(f, "block signature is invalid"),
            Self::UnauthorizedBlockAuthor => {
                write!(f, "block author is not an authorized authority")
            }
            Self::InvalidTransactionSignature => write!(f, "transaction signature is invalid"),
            Self::DuplicateDeviceId(id) => write!(f, "device {id} already registered"),
            Self::UnknownDevice(id) => write!(f, "device {id} not found on chain"),
            Self::UnauthorizedSigner => {
                write!(f, "transaction signer is not authorized for this operation")
            }
            Self::InvalidNonce { expected, got } => {
                write!(f, "invalid nonce: expected {expected}, got {got}")
            }
            Self::SerializationError(msg) => write!(f, "serialization error: {msg}"),
            Self::InvalidKey(msg) => write!(f, "invalid key: {msg}"),
            Self::InvalidAttestation(msg) => {
                write!(f, "invalid identity attestation: {msg}")
            }
        }
    }
}

impl std::error::Error for ChainError {}
