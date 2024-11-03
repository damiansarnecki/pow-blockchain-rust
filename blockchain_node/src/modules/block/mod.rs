use super::crypto::hash;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeaders {
    pub previous_hash: [u8; 32],
    pub nonce: u64,
    pub difficulty: u32,
    pub number: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub headers: BlockHeaders,
    pub hash: [u8; 32],
}

impl Block {
    pub fn new(previous_hash: &[u8; 32], difficulty: u32, number: u32) -> Block {
        let headers = BlockHeaders {
            previous_hash: previous_hash.clone(),
            nonce: 0,
            difficulty: difficulty,
            number: number,
        };

        Block {
            headers: headers,
            hash: [0u8; 32],
        }
    }

    pub fn calculate_hash(&self) -> [u8; 32] {
        let serialized = serde_json::to_string(&self.headers).expect("Serialization failed");
        hash(&serialized)
    }

    pub fn genesis() -> Block {
        Block::new(&[0u8; 32], 100, 0)
    }
}
