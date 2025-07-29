use std::collections::HashMap;

use super::TexturedBlockType;
use shared::entities::BlockID;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct RenderBlockTypeStorage {
    block_types: HashMap<BlockID, TexturedBlockType>,
}

impl RenderBlockTypeStorage {
    pub fn new(block_types: HashMap<BlockID, TexturedBlockType>) -> RenderBlockTypeStorage {
        RenderBlockTypeStorage { block_types }
    }

    #[allow(dead_code)]
    pub fn set_block_type(&mut self, block_id: BlockID, block_type: TexturedBlockType) {
        self.block_types.insert(block_id, block_type);
    }

    pub fn get_block_type_from_id(&self, id: BlockID) -> Option<&TexturedBlockType> {
        self.block_types.get(&id)
    }
}
