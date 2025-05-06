use std::collections::HashMap;

use super::MeshBlockType;
use shared::entities::BlockID;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BlockTypeStorage {
    block_types: HashMap<BlockID, MeshBlockType>,
}

impl BlockTypeStorage {
    pub fn new(block_types: HashMap<BlockID, MeshBlockType>) -> BlockTypeStorage {
        BlockTypeStorage { block_types }
    }

    pub fn set_block_type(&mut self, block_id: BlockID, block_type: MeshBlockType) {
        self.block_types.insert(block_id, block_type);
    }

    pub fn get_block_type_from_id(&self, id: BlockID) -> Option<&MeshBlockType> {
        self.block_types.get(&id)
    }
}
