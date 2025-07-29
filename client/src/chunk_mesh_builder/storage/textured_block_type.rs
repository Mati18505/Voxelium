use std::collections::HashMap;
use shared::entities::BlockSide;

use crate::chunk_mesh_builder::{MaterialName, RenderBlockType, RenderShape, TextureName};

#[derive(Debug, Clone, PartialEq)]
pub struct TexturedBlockType {
    pub block_type: String,

    pub is_visible: bool,
    pub is_translucent: bool,
    pub material_name: String,

    render_shape: RenderShape,
    top_texture: Option<String>,
    side_texture: String,
    bottom_texture: Option<String>,
}

impl RenderBlockType for TexturedBlockType {
    fn get_server_block_type_name(&self) -> &str {
        &self.block_type
    }

    fn visible(&self) -> bool {
        self.is_visible
    }

    fn translucent(&self) -> bool {
        self.is_translucent
    }

    fn get_render_shape(&self) -> &super::RenderShape {
        &self.render_shape
    }
}

impl Default for TexturedBlockType {
    fn default() -> Self {
        let side_texture = "default".to_owned();

        Self {
            block_type: "none".to_owned(),
            is_visible: false,
            is_translucent: false,
            material_name: "default".to_owned(),

            top_texture: None,
            side_texture: side_texture.clone(),
            bottom_texture: None,
            render_shape: RenderShape::create_textured_cube(side_texture, HashMap::default()),
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
        let mut result = self.block_type;

        Self::create_render_shape(&mut result);

        result
    }

    fn create_render_shape(textured_block_type: &mut TexturedBlockType) {
        let mut textures = HashMap::<BlockSide, TextureName>::default();

        if let Some(top_texture) = textured_block_type.top_texture.clone() {
            textures.insert(BlockSide::Top, top_texture);
        }
        if let Some(bottom_texture) = textured_block_type.bottom_texture.clone() {
            textures.insert(BlockSide::Top, bottom_texture);
        }

        textured_block_type.render_shape = RenderShape::create_textured_cube(textured_block_type.side_texture.clone(), textures);
    }
}
