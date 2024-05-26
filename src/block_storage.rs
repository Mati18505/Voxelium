use crate::types::CHUNK_SIZE;
use crate::types::{BlockID, BlockPos};
use cgmath::Vector3;
use crate::shared;

#[derive(Debug)]
pub struct BlockStorage {
    block_types: Vec<BlockID>,
}

impl BlockStorage {
    pub fn new(blocks: Vec<BlockID>) -> Self {
        BlockStorage {
            block_types: blocks
        }
    }

    pub fn get_block(&self, pos: Vector3<usize>) -> BlockID {
        let idx = shared::index(pos, CHUNK_SIZE);
        self.block_types[idx]
    }
}

impl Clone for BlockStorage {
    fn clone(&self) -> Self {
        BlockStorage {
            block_types: self.block_types.clone()
        }
    }
} 