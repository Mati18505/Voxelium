use shared::entities::BlockSide;

use crate::chunk_mesh_builder::MaterialName;

#[derive(Debug, Clone, PartialEq)]
pub struct TexturedBlockType {
    pub block_type: String,

    pub is_visible: bool,
    pub is_translucent: bool,
    pub material_name: String,

    top_texture: Option<String>,
    side_texture: String,
    bottom_texture: Option<String>,
}

impl TexturedBlockType {
    pub fn get_block_side_texture(&self, side: BlockSide) -> &str {
        match side {
            BlockSide::Top => self.top_texture.as_deref().unwrap_or(&self.side_texture),
            BlockSide::Bottom => self.bottom_texture.as_deref().unwrap_or(&self.side_texture),
            _ => &self.side_texture,
        }
    }
}

impl Default for TexturedBlockType {
    fn default() -> Self {
        Self {
            block_type: "none".to_owned(),
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
pub struct TexturedBlockTypeBuilder {
    block_type: TexturedBlockType,
}

impl TexturedBlockTypeBuilder {
    pub fn new(block_type: &str) -> TexturedBlockTypeBuilder {
        let textured_block_type = TexturedBlockType {
            block_type: block_type.to_owned(),
            ..Default::default()
        };

        TexturedBlockTypeBuilder {
            block_type: textured_block_type,
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

    pub fn build(self) -> TexturedBlockType {
        self.block_type
    }
}
