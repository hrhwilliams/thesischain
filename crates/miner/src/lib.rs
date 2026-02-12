#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod chain;
mod crypto;
mod error;
mod genesis;
pub mod http;
pub mod network;
mod state;
mod types;

pub use chain::Chain;
pub use crypto::{sign_attestation, sign_transaction, verify_attestation};
pub use error::ChainError;
pub use genesis::{GenesisDevice, create_genesis};
pub use state::{DeviceRecord, KeyDirectory};
pub use types::{Block, BlockHeader, ChainId, IdentityAttestation, SignedTransaction, Transaction};

#[cfg(test)]
mod tests;
