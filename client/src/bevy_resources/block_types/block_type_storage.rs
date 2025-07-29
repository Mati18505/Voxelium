use std::{collections::HashMap, fmt::Debug};

use shared::entities::BlockID;

use crate::bevy_resources::RenderBlockType;

#[derive(Debug, Default)]
pub struct RenderBlockTypeStorage {
    block_types: HashMap<BlockID, RenderBlockType>,
}

impl RenderBlockTypeStorage {
    pub fn new(block_types: HashMap<BlockID, RenderBlockType>) -> RenderBlockTypeStorage {
        RenderBlockTypeStorage { block_types }
    }

    #[allow(dead_code)]
    pub fn set_block_type(&mut self, block_id: BlockID, block_type: RenderBlockType) {
        self.block_types.insert(block_id, block_type);
    }

    pub fn get_block_type_from_id(&self, id: BlockID) -> Option<RenderBlockType> {
        self.block_types.get(&id)
    }
}
