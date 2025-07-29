use std::{
    collections::HashMap,
    fmt::{self, Debug},
};

use shared::entities::BlockID;

use crate::chunk_mesh_builder::RenderShape;

pub trait RenderBlockType: Send + Sync + Debug {
    fn get_server_block_type_name(&self) -> &str;
    fn visible(&self) -> bool;
    fn translucent(&self) -> bool;
    fn get_render_shape(&self) -> &RenderShape;
}

#[derive(Debug, Default)]
pub struct RenderBlockTypeStorage {
    block_types: HashMap<BlockID, Box<dyn RenderBlockType>>,
}

impl RenderBlockTypeStorage {
    pub fn new(block_types: HashMap<BlockID, Box<dyn RenderBlockType>>) -> RenderBlockTypeStorage {
        RenderBlockTypeStorage { block_types }
    }

    #[allow(dead_code)]
    pub fn set_block_type(&mut self, block_id: BlockID, block_type: Box<dyn RenderBlockType>) {
        self.block_types.insert(block_id, block_type);
    }

    pub fn get_block_type_from_id(&self, id: BlockID) -> Option<&dyn RenderBlockType> {
        self.block_types.get(&id).map(|t| t.as_ref())
    }
}
