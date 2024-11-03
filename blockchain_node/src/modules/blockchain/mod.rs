use super::{block::Block, storage::BlockStorage};
use std::collections::HashMap;

#[derive(Clone)]
pub struct BlockIndex {
    pub hash: [u8; 32],
    pub number: u32,
    pub prev_block_hash: [u8; 32],
    pub work_sum: u32,
    pub difficulty: u32,
    pub db_key: u32,
}

pub struct Blockchain {
    pub block_index_map: HashMap<[u8; 32], BlockIndex>,
    pub orphan_blocks_map: HashMap<[u8; 32], Block>,
    pub current_chain: [u8; 32],
    pub block_db: BlockStorage,
}

impl Blockchain {
    pub fn new(db_path: &str) -> Blockchain {
        let block_db = BlockStorage::new(db_path);
        let block_index_map: HashMap<[u8; 32], BlockIndex> = HashMap::new();
        let orphan_blocks_map: HashMap<[u8; 32], Block> = HashMap::new();

        let current_chain = [0u8; 32];

        Blockchain {
            block_db,
            block_index_map,
            orphan_blocks_map,
            current_chain,
        }
    }

    pub fn add_block_to_index(&mut self, block: &Block, db_key: u32) -> BlockIndex {
        let prev_block_work_sum = {
            match self.get_block_index(&block.headers.previous_hash) {
                Some(block_index) => block_index.work_sum,
                None => 0,
            }
        };

        let work_sum = prev_block_work_sum + block.headers.difficulty;

        let new_index = BlockIndex {
            number: block.headers.number,
            hash: block.hash,
            prev_block_hash: block.headers.previous_hash,
            difficulty: block.headers.difficulty,
            work_sum: work_sum,
            db_key: db_key,
        };

        self.block_index_map.insert(block.hash, new_index.clone());

        let prev_block_work_sum = self.get_block_index(&self.current_chain).unwrap().work_sum;

        if work_sum > prev_block_work_sum {
            self.current_chain = block.hash;
        }

        new_index
    }

    pub fn add_orphan_block(&mut self, block: &Block) {
        self.orphan_blocks_map
            .insert(block.hash, block.clone())
            .unwrap();
    }

    pub fn initialize(&mut self) {
        self.initialize_block_index();

        if self.block_index_map.is_empty() {
            let genesis_block = Block::genesis();
            let genesis_db_key = self.block_db.save_block(&genesis_block);
            self.add_block_to_index(&genesis_block, genesis_db_key);
        }
    }

    fn initialize_block_index(&mut self) {
        let mut biggest_work = 0;

        for entry in self.block_db.get_iter() {
            let (_key, value) = entry.expect("Error reading from database.");

            let block: Block =
                serde_json::from_slice(&value).expect("Failed to deserialize block.");
            let block_key =
                u32::from_be_bytes(_key.as_ref().try_into().expect("Key length mismatch"));
            println!("{}", block_key);
            let index = self.add_block_to_index(&block, block_key);

            if (index.work_sum > biggest_work) {
                self.current_chain = block.hash;
                biggest_work = index.work_sum;
            }
        }
    }

    pub fn set_longest_chain(&mut self, new_chain_hash: [u8; 32]) {
        self.current_chain = new_chain_hash;
    }

    pub fn get_block_index(&self, hash: &[u8; 32]) -> Option<&BlockIndex> {
        self.block_index_map.get(hash)
    }

    pub fn connect_orphans(&mut self, parent_hash: [u8; 32]) {
        let mut parents_stack = vec![parent_hash];

        while let Some(parent_hash) = parents_stack.pop() {
            let mut orphans_to_connect = vec![];

            for (key, orphan) in self.orphan_blocks_map.iter() {
                if (orphan.headers.previous_hash == parent_hash) {
                    orphans_to_connect.push(*key);
                }
            }

            for orphan_hash in orphans_to_connect.iter() {
                if let Some(orphan_block) = self.orphan_blocks_map.remove(orphan_hash) {
                    let db_key = self.block_db.save_block(&orphan_block);
                    self.add_block_to_index(&orphan_block, db_key);

                    parents_stack.push(orphan_block.hash);
                }
            }
        }
    }

    pub fn get_last_block_index(&self) -> Option<&BlockIndex> {
        self.block_index_map.get(&self.current_chain)
    }
}
