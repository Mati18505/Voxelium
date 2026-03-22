use crate::entities::CHUNK_SIZE;

use super::{types::BlockID, BlockInChunkPos};

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStorage {
    block_types: Vec<BlockID>,
}

impl BlockStorage {
    pub fn new(blocks: Vec<BlockID>) -> Self {
        BlockStorage {
            block_types: blocks,
        }
    }

    pub fn get_block(&self, pos: BlockInChunkPos) -> BlockID {
        self.block_types[pos.index()]
    }

    pub fn set_block(&mut self, pos: BlockInChunkPos, new_block: BlockID) {
        self.block_types[pos.index()] = new_block;
    }

    pub fn get_blocks(&self) -> &[BlockID] {
        &self.block_types
    }

    pub fn iter(&self) -> std::slice::Iter<'_, BlockID> {
        self.block_types.iter()
    }
}

