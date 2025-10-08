use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub struct Peer {
    pub address: String,
}
