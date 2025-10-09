use blake3::Hash;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Block {
    block: UnhashedBlock,
    hash: Hash,
}

#[derive(Serialize, Deserialize)]
pub struct UnhashedBlock {
    items: Vec<SignedBlockItem>,
    timestamp: String,
    parent: Hash,
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