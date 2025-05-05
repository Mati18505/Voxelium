use super::{BlockID, BlockType};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BlockTypeStorage {
    block_types: Vec<BlockType>,
}

impl BlockTypeStorage {
    pub fn new(block_types: Vec<BlockType>) -> Self {
        assert!(block_types.len() <= BlockID::MAX as usize);

        BlockTypeStorage { block_types }
    }

    pub fn get_by_id(&self, id: BlockID) -> Option<&BlockType> {
        self.block_types.get(id as usize)
    }

    pub fn add_block_type(&mut self, block_type: BlockType) {
        assert!(self.block_types.len() < BlockID::MAX as usize);

        self.block_types.push(block_type);
    }
}