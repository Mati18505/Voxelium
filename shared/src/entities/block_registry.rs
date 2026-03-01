use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;

use super::BlockID;

static BLOCK_NAME_TO_ID: OnceCell<Arc<HashMap<String, BlockID>>> = OnceCell::new();

pub fn init_block_names(blocks: Vec<(String, BlockID)>) {
    let map = blocks.into_iter().collect();

    BLOCK_NAME_TO_ID
        .set(Arc::new(map))
        .expect("BLOCK_NAME_TO_ID already initialized");
}

pub fn name_to_block_id(name: &str) -> BlockID {
    let map = BLOCK_NAME_TO_ID
        .get()
        .expect("BLOCK_NAME_TO_ID not initialized");

    map.get(name).copied().unwrap_or_else(|| {
        assert!(name != "air", "Missing air block");

        BlockID::default()
    })
}
