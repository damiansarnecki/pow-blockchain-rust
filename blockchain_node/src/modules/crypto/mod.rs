use keccak256::keccak256_hash;
use serde::Serialize;

pub mod keccak256;

pub fn hash<T: Serialize>(obj: &T) -> [u8; 32] {
    let serialized = serde_json::to_string(&obj).expect("Serialization failed");
    keccak256_hash(&serialized)
}

pub fn to_hex_string(hash: &[u8; 32]) -> String {
    let hex_string: String = hash.iter().map(|byte| format!("{:02x}", byte)).collect();
    format!("0x{}", hex_string)
}