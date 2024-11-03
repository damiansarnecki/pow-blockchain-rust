use tiny_keccak::{Keccak, Hasher};

pub fn keccak256_hash(input: &str) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(input.as_bytes());
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    
    output 
}
