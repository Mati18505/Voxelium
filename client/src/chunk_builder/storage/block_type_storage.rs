use std::collections::HashMap;

use super::MeshBlockType;
use shared::entities::BlockID;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BlockTypeStorage {
    block_types: HashMap<BlockID, MeshBlockType>,
}

impl BlockTypeStorage {
    pub fn get_block_type_from_id(&self, id: BlockID) -> Option<&MeshBlockType> {
        self.block_types.get(&id)
    }
}
