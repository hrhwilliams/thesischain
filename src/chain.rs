use serde::{Deserialize, Serialize};
use serde_with::hex::Hex;
use serde_with::serde_as;
use std::fmt::Debug;

const STRENGTH: usize = 3;

pub type NodeContent = String;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node<S>
where
    S: Serialize,
{
    pub message: S,
    #[serde_as(as = "Option<Hex>")]
    pub parent: Option<[u8; blake3::OUT_LEN]>,
    #[serde_as(as = "Hex")]
    pub hash: [u8; blake3::OUT_LEN],
    pub timestamp: String,
    pub nonce: usize,
}

impl<S> PartialEq for Node<S>
where
    S: Serialize,
{
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl<S> Node<S>
where
    S: Serialize,
{
    /// Mines a new genesis block (no parent).
    pub fn new(message: S) -> Self {
        let now = chrono::Utc::now();
        let timestamp = now.to_rfc3339();

        let (hash, nonce) = Self::mine(&message, &now, None);

        Self {
            message,
            parent: None,
            hash,
            timestamp,
            nonce,
        }
    }

    /// Mines a new block that appends to a parent block.
    pub fn append(message: S, parent: &Node<S>) -> Self {
        let now = chrono::Utc::now();
        let timestamp = now.to_rfc3339();

        let (hash, nonce) = Self::mine(&message, &now, Some(&parent.hash));


        Self {
            message,
            parent: Some(parent.hash),
            hash,
            timestamp,
            nonce,
        }
    }

    /// Private helper function to perform the proof-of-work mining.
    fn mine(
        message: &S,
        now: &chrono::DateTime<chrono::Utc>,
        parent_hash: Option<&[u8; blake3::OUT_LEN]>,
    ) -> ([u8; blake3::OUT_LEN], usize) {
        let mut nonce: usize = 0;
        let message_bytes = serde_json::to_vec(message).unwrap();

        loop {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&message_bytes);
            if let Some(p_hash) = parent_hash {
                hasher.update(p_hash);
            }
            hasher.update(&now.to_rfc3339().as_bytes());
            hasher.update(&nonce.to_le_bytes());

            let hash = hasher.finalize();
            let (_left, right) = hash.as_bytes().split_at(blake3::OUT_LEN - STRENGTH);

            if right.iter().all(|&b| b == 0) {
                return (hash.into(), nonce);
            } else {
                nonce += 1;
            }
        }
    }

    /// Checks if a block is valid by verifying its proof-of-work.
    pub fn is_valid_pow(&self) -> bool {
        let message_bytes = serde_json::to_vec(&self.message).unwrap();
        let mut hasher = blake3::Hasher::new();
        hasher.update(&message_bytes);
        if let Some(p_hash) = self.parent {
            hasher.update(&p_hash);
        }
        hasher.update(&self.nonce.to_le_bytes());
        let hash = hasher.finalize();

        if hash.as_bytes() != &self.hash {
            return false;
        }

        let (_left, right) = hash.as_bytes().split_at(blake3::OUT_LEN - STRENGTH);
        right.iter().all(|&b| b == 0)
    }

    /// Checks if a new block is a valid successor to the current one.
    pub fn is_valid(&self, parent: &Node<S>) -> bool {
        if self.parent != Some(parent.hash) {
            return false;
        }
        self.is_valid_pow()
    }

    /// Validates an entire chain.
    pub fn validate_chain(chain: &[Node<S>]) -> bool {
        if chain.is_empty() {
            return true;
        }
        // Check genesis block
        if !chain[0].is_valid_pow() {
            log::warn!("Genesis block has invalid PoW");
            return false;
        }
        // Check subsequent blocks
        for i in 1..chain.len() {
            if !chain[i].is_valid(&chain[i - 1]) {
                log::warn!("Invalid block at index {}", i);
                return false;
            }
        }
        true
    }
}

// sorta like the routes thing from CMU that lets you see a host through different routes to see if your route has been tampered with
