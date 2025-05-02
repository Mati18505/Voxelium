use crate::entities::CHUNK_SIZE;

use super::{types::BlockID, BlockInChunkPos};

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStorage {
    block_types: Vec<BlockID>,
}

impl BlockStorage {
    pub fn new(blocks: Vec<BlockID>) -> Self {
        assert!(blocks.len() == CHUNK_SIZE.pow(3));

        BlockStorage {
            block_types: blocks,
        }
    }

    pub fn get_block(&self, pos: BlockInChunkPos) -> BlockID {
        self.block_types[pos.index()]
    }

    pub fn get_blocks(&self) -> &[BlockID] {
        &self.block_types
    }

    pub fn iter(&self) -> std::slice::Iter<BlockID> {
        self.block_types.iter()
    }
}

impl Default for BlockStorage {
    fn default() -> Self {
        Self::new(vec![0; CHUNK_SIZE.pow(3)])
    }
}
