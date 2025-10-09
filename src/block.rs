use blake3::Hash;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Block {
    items: Vec<SignedBlockItem>,
    timestamp: String,
    parent: Hash,
    hash: Hash,
    nonce: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SignedBlockItem {
    item: BlockItem,
    source: String,
}

#[derive(Serialize, Deserialize)]
pub enum BlockItem {
    Text(String),
}
