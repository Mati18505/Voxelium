use std::{collections::HashMap, fmt::Debug};

use bevy::render::render_resource::Texture;
use shared::entities::BlockID;

use crate::{
    bevy_resources::{RenderBlockType, RenderShapeStorage, TextureIndexDictionary, TextureName},
    chunk_mesh_builder::RenderShape,
};

#[derive(Debug, Default)]
pub struct RenderBlockTypeStorage {
    block_types: Vec<RenderBlockType>,
}

impl RenderBlockTypeStorage {
    pub fn new(block_types: Vec<RenderBlockType>) -> RenderBlockTypeStorage {
        RenderBlockTypeStorage { block_types }
    }

    pub fn get_block_type_from_id(&self, id: BlockID) -> Option<&RenderBlockType> {
        self.block_types.get(id as usize)
    }

    pub fn compile(&self, texture_dictionary: &TextureIndexDictionary) -> RenderShapeStorage {
        let compiled: Vec<RenderShape> = self
            .block_types
            .iter()
            .map(|block_type| block_type.render_desc.compile(texture_dictionary))
            .collect();

        RenderShapeStorage::new(compiled)
    }
}
