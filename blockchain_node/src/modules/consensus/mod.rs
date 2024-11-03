use super::block::Block;
use crate::{modules::network::broadcast_inv, SharedPeers, BLOCKCHAIN};
use num_bigint::BigUint;
use rand::Rng;
use std::sync::Arc;

pub fn start_mining_loop(peers: SharedPeers) {
    loop {
        let (highest_block_hash, highest_block_number) = {
            let blockchain = BLOCKCHAIN.lock().unwrap();
            let prev_block_hash = blockchain.current_chain;
            let prev_block_index = blockchain.get_block_index(&prev_block_hash).unwrap();

            let next_block_number = prev_block_index.number;

            (prev_block_hash, next_block_number)
        };

        let next_block = mine_block(&highest_block_hash, highest_block_number + 1, 50000);

        let hash_exists_in_db = {
            let blockchain = BLOCKCHAIN.lock().unwrap();

            match blockchain.get_block_index(&next_block.headers.previous_hash) {
                Some(_) => true,
                None => false,
            }
        };
        println!("Mined block {}!", next_block.headers.number);

        if highest_block_hash == next_block.headers.previous_hash && hash_exists_in_db {
            let mut blockchain_lock = BLOCKCHAIN.lock().unwrap();
            let mined_block_key = blockchain_lock.block_db.save_block(&next_block);
            blockchain_lock.add_block_to_index(&next_block, mined_block_key);
            broadcast_inv(&next_block, Arc::clone(&peers));
        }
    }
}

pub fn validate_block(block: &Block, required_difficulty: u32) -> Result<(), String> {
    if !validate_block_hash(block) {
        return Err("Block hash invalid".into());
    }

    if !validate_hash_difficulty(&block.hash, required_difficulty) {
        return Err("Hash difficulty too low".into());
    }

    Ok(())
}

fn validate_block_hash(block: &Block) -> bool {
    block.calculate_hash() == block.hash
}

pub fn mine_block(prev_hash: &[u8; 32], number: u32, difficulty: u32) -> Block {
    let mut rng = rand::thread_rng();
    let mut new_block = Block::new(prev_hash, difficulty, number);

    new_block.headers.nonce = rng.gen::<u64>();

    loop {
        new_block.hash = new_block.calculate_hash();

        if validate_block(&new_block, difficulty).is_ok() {
            return new_block;
        }

        new_block.headers.nonce = new_block.headers.nonce.wrapping_add(1);
    }
}

fn validate_hash_difficulty(hash: &[u8], required_difficulty: u32) -> bool {
    let max_target = BigUint::from_bytes_be(&[0xff; 32]);
    let hash_value = BigUint::from_bytes_be(hash);
    let target = max_target / BigUint::from(required_difficulty);

    hash_value <= target
}
