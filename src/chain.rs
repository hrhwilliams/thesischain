use serde::{Deserialize, Serialize};
use serde_with::hex::Hex;
use serde_with::serde_as;

const STRENGTH: usize = 3;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Node {
    pub message: String,
    #[serde_as(as = "Option<Hex>")]
    pub parent: Option<[u8; blake3::OUT_LEN]>,
    #[serde_as(as = "Hex")]
    pub hash: [u8; blake3::OUT_LEN],
    pub nonce: usize,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Node {
    pub fn new(message: &str) -> Self {
        let mut nonce: usize = 0;

        loop {
            let hash = blake3::Hasher::new()
                .update(message.as_bytes())
                .update(&nonce.to_le_bytes())
                .finalize();

            let (_left, right) = hash.as_bytes().split_at(blake3::OUT_LEN - STRENGTH);

            if right.iter().all(|&b| b == 0) {
                return Self {
                    message: message.to_string(),
                    parent: None,
                    hash: hash.into(),
                    nonce,
                };
            } else {
                nonce += 1;
            }
        }
    }

    pub fn append(message: &str, parent: &Node) -> Self {
        let mut nonce: usize = 0;

        loop {
            let hash = blake3::Hasher::new()
                .update(message.as_bytes())
                .update(&parent.hash)
                .update(&nonce.to_le_bytes())
                .finalize();

            let (_left, right) = hash.as_bytes().split_at(blake3::OUT_LEN - STRENGTH);

            if right.iter().all(|&b| b == 0) {
                return Self {
                    message: message.to_string(),
                    parent: Some(parent.hash.clone()),
                    hash: hash.into(),
                    nonce,
                };
            } else {
                nonce += 1;
            }
        }
    }
}

// sorta like the routes thing from CMU that lets you see a host through different routes to see if your route has been tampered with
