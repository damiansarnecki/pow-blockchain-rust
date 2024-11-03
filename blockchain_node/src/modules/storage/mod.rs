use super::block::Block;
use sled::{Db, Iter};

pub struct BlockStorage {
    db: Db,
    counter: u32,
}

impl BlockStorage {
    pub fn new(db_path: &str) -> BlockStorage {
        let db = sled::open(db_path).expect("Can't connect to database.");
        let counter = db.iter().count() as u32;

        BlockStorage { db, counter }
    }

    pub fn save_block(&mut self, block: &Block) -> u32 {
        let serialized_block = serde_json::to_vec(&block).unwrap();
        let new_block_key: u32 = self.counter;

        self.db
            .insert(new_block_key.to_be_bytes(), serialized_block)
            .unwrap();
        
        self.increase_counter();
        new_block_key
    }

    pub fn read_block(&self, key: u32) -> Option<Block> {
        match self.db.get(key.to_be_bytes()) {
            Ok(Some(serialized_block)) => {
                let block: Block = serde_json::from_slice(&serialized_block).unwrap();
                Some(block)
            }
            Ok(None) => None,
            Err(e) => {
                println!("{}", e);
                None
            }
        }
    }

    pub fn get_iter(&self) -> Iter {
        self.db.iter()
    }

    fn increase_counter(&mut self) {
        self.counter += 1;
    }
}
