use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use uuid::Uuid;

use crate::error::ChainError;
use crate::types::{Block, BlockHeader, IdentityAttestation, SignedTransaction, Transaction};

/// SHA-256 hash of arbitrary bytes.
pub fn hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// SHA-256 hash of a bincode-serialized block.
pub fn hash_block(block: &Block) -> Result<[u8; 32], ChainError> {
    let encoded = bincode::serde::encode_to_vec(block, bincode::config::standard())
        .map_err(|e| ChainError::SerializationError(e.to_string()))?;
    Ok(hash_bytes(&encoded))
}

/// Builds the message that a block author signs:
/// `index || timestamp || previous_hash || transactions_hash`
fn block_signing_message(
    index: u64,
    timestamp: u64,
    previous_hash: &[u8; 32],
    transactions_hash: &[u8; 32],
) -> Vec<u8> {
    let mut msg = Vec::with_capacity(8 + 8 + 32 + 32);
    msg.extend_from_slice(&index.to_le_bytes());
    msg.extend_from_slice(&timestamp.to_le_bytes());
    msg.extend_from_slice(previous_hash);
    msg.extend_from_slice(transactions_hash);
    msg
}

/// Computes the SHA-256 hash of the bincode-serialized transactions vector.
pub fn hash_transactions(txs: &[SignedTransaction]) -> Result<[u8; 32], ChainError> {
    let encoded = bincode::serde::encode_to_vec(txs, bincode::config::standard())
        .map_err(|e| ChainError::SerializationError(e.to_string()))?;
    Ok(hash_bytes(&encoded))
}

/// Builds the message that a transaction signer signs:
/// bincode-serialized `(payload, nonce)`.
fn transaction_signing_message(payload: &Transaction, nonce: u64) -> Result<Vec<u8>, ChainError> {
    bincode::serde::encode_to_vec((payload, nonce), bincode::config::standard())
        .map_err(|e| ChainError::SerializationError(e.to_string()))
}

/// Sign a transaction, producing a `SignedTransaction`.
pub fn sign_transaction(
    payload: Transaction,
    nonce: u64,
    signing_key: &SigningKey,
) -> Result<SignedTransaction, ChainError> {
    let msg = transaction_signing_message(&payload, nonce)?;
    let signature = signing_key.sign(&msg);

    Ok(SignedTransaction {
        payload,
        signer: signing_key.verifying_key().to_bytes(),
        signature,
        nonce,
    })
}

/// Verify a signed transaction's signature.
pub fn verify_transaction(tx: &SignedTransaction) -> Result<(), ChainError> {
    let verifying_key =
        VerifyingKey::from_bytes(&tx.signer).map_err(|e| ChainError::InvalidKey(e.to_string()))?;
    let msg = transaction_signing_message(&tx.payload, tx.nonce)?;

    verifying_key
        .verify(&msg, &tx.signature)
        .map_err(|_| ChainError::InvalidTransactionSignature)
}

/// Create and sign a block from its components.
pub fn sign_block(
    index: u64,
    timestamp: u64,
    previous_hash: [u8; 32],
    transactions: Vec<SignedTransaction>,
    signing_key: &SigningKey,
) -> Result<Block, ChainError> {
    let transactions_hash = hash_transactions(&transactions)?;
    let msg = block_signing_message(index, timestamp, &previous_hash, &transactions_hash);
    let signature = signing_key.sign(&msg);

    Ok(Block {
        header: BlockHeader {
            index,
            timestamp,
            previous_hash,
            transactions_hash,
            author: signing_key.verifying_key().to_bytes(),
            signature,
        },
        transactions,
    })
}

/// Builds the message that the backend signs for an identity attestation:
/// `bincode(user_id, device_id, issued_at)`.
fn attestation_signing_message(
    user_id: Uuid,
    device_id: Uuid,
    issued_at: u64,
) -> Result<Vec<u8>, ChainError> {
    bincode::serde::encode_to_vec((user_id, device_id, issued_at), bincode::config::standard())
        .map_err(|e| ChainError::SerializationError(e.to_string()))
}

/// Create a signed identity attestation.
pub fn sign_attestation(
    user_id: Uuid,
    device_id: Uuid,
    issued_at: u64,
    signing_key: &SigningKey,
) -> Result<IdentityAttestation, ChainError> {
    let msg = attestation_signing_message(user_id, device_id, issued_at)?;
    let signature = signing_key.sign(&msg);
    Ok(IdentityAttestation {
        user_id,
        device_id,
        issued_at,
        backend_key: signing_key.verifying_key().to_bytes(),
        signature,
    })
}

/// Verify an identity attestation against a known backend public key.
pub fn verify_attestation(
    attestation: &IdentityAttestation,
    expected_backend_key: &VerifyingKey,
) -> Result<(), ChainError> {
    if attestation.backend_key != expected_backend_key.to_bytes() {
        return Err(ChainError::InvalidAttestation(
            "backend key mismatch".into(),
        ));
    }
    let msg = attestation_signing_message(
        attestation.user_id,
        attestation.device_id,
        attestation.issued_at,
    )?;
    expected_backend_key
        .verify(&msg, &attestation.signature)
        .map_err(|_| ChainError::InvalidAttestation("signature invalid".into()))
}

/// Verify a block header's signature and transactions hash.
pub fn verify_block(block: &Block) -> Result<(), ChainError> {
    // Verify transactions hash
    let computed_hash = hash_transactions(&block.transactions)?;
    if computed_hash != block.header.transactions_hash {
        return Err(ChainError::InvalidTransactionsHash);
    }

    // Verify block signature
    let verifying_key = VerifyingKey::from_bytes(&block.header.author)
        .map_err(|e| ChainError::InvalidKey(e.to_string()))?;
    let msg = block_signing_message(
        block.header.index,
        block.header.timestamp,
        &block.header.previous_hash,
        &block.header.transactions_hash,
    );

    verifying_key
        .verify(&msg, &block.header.signature)
        .map_err(|_| ChainError::InvalidBlockSignature)
}
