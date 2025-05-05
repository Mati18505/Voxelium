use shared::entities::{BlockSide, BlockType};

use crate::chunk_builder::MaterialName;

#[derive(Debug, Clone, PartialEq)]
pub struct MeshBlockType {
    pub block_type: BlockType,

    pub is_visible: bool,
    pub is_translucent: bool,
    pub material_name: String,

    top_texture: Option<String>,
    side_texture: String,
    bottom_texture: Option<String>,
}

impl MeshBlockType {
    pub fn get_block_side_texture<'a>(&'a self, side: BlockSide) -> &'a str {
        match side {
            BlockSide::Top => self.top_texture.as_deref().unwrap_or(&self.side_texture),
            BlockSide::Bottom => self.bottom_texture.as_deref().unwrap_or(&self.side_texture),
            _ => &self.side_texture,
        }
    }
}

impl Default for MeshBlockType {
    fn default() -> Self {
        Self {
            block_type: BlockType::default(),
            is_visible: false,
            is_translucent: false,
            material_name: "default".to_owned(),

            top_texture: None,
            side_texture: "default".to_owned(),
            bottom_texture: None,
        }
    }
}

#[derive(Default)]
pub struct MeshBlockTypeBuilder {
    block_type: MeshBlockType,
}

impl MeshBlockTypeBuilder {
    pub fn new(block_type: BlockType) -> MeshBlockTypeBuilder {
        let mut mesh_block_type = MeshBlockType::default();
        mesh_block_type.block_type = block_type;

        MeshBlockTypeBuilder {
            block_type: mesh_block_type,
        }
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.block_type.is_visible = visible;
        self
    }

    pub fn translucent(mut self, translucent: bool) -> Self {
        self.block_type.is_translucent = translucent;
        self
    }

    pub fn material(mut self, material_name: MaterialName) -> Self {
        self.block_type.material_name = material_name;
        self
    }

    pub fn texture(mut self, block_side: BlockSide, texture_name: &str) -> Self {
        use BlockSide::*;

        match block_side {
            Back | Front | Left | Right => self.block_type.side_texture = texture_name.to_owned(),
            Top => self.block_type.top_texture = Some(texture_name.to_owned()),
            Bottom => self.block_type.bottom_texture = Some(texture_name.to_owned()),
        }
        self
    }

    pub fn build(self) -> MeshBlockType {
        self.block_type
    }
}
