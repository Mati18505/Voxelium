use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;

use super::BlockID;

static BLOCK_NAME_TO_ID: Lazy<RwLock<HashMap<String, BlockID>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

pub fn init_block_names(blocks: Vec<(String, BlockID)>) {
    let mut map = BLOCK_NAME_TO_ID.write().unwrap();

    for (name, id) in blocks {
        map.insert(name, id);
    }
}

pub fn name_to_block_id(name: &str) -> BlockID {
    let map = BLOCK_NAME_TO_ID.read().unwrap();

    if let Some(block_id) = map.get(name).copied() {
        return block_id
    } else {
        if name == "air" {
            assert!(true, "Missing air block");
        } 

        BlockID::default()
    }
}