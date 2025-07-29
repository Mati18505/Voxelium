use std::default;

use bevy::render::render_resource::Texture;
use shared::entities::VoxelColor;

use crate::{
    bevy_resources::{texture_dictionary, MaterialName, TextureDictionary},
    chunk_mesh_builder::RenderShape,
};

#[derive(Debug)]
pub struct RenderData {
    pub visible: bool,
    pub translucent: bool,
    pub material: MaterialName,
}

impl Default for RenderData {
    fn default() -> Self {
        Self {
            visible: true,
            translucent: false,
            material: "default".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum RenderDesc {
    TexturedCube {
        render_data: RenderData,
        textured_cube_desc: TexturedCubeDesc,
    },
    ColoredCube {
        render_data: RenderData,
        palette: Vec<VoxelColor>,
    },
    Invisible,
}

impl RenderDesc {
    pub fn compile(&self, texture_dictionary: TextureDictionary) -> RenderShape {
        match self {
            RenderDesc::TexturedCube => self.compile_textured_cube(texture_dictionary),
            RenderDesc::ColoredCube => self.compile_colored_cube(),
            RenderDesc::Invisible => RenderShape::Invisible,
        }
    }

    fn compile_textured_cube(&self, texture_dictionary: TextureDictionary) -> RenderShape {
        todo!();
        /*
        let render_data = VoxelRenderData {
            visible: textured_block_type.visible(),
            material: 0,
        };

        let mut textures = HashMap::<BlockSide, TextureName>::default();

        if let Some(top_texture) = textured_block_type.top_texture.clone() {
            textures.insert(BlockSide::Top, top_texture);
        }
        if let Some(bottom_texture) = textured_block_type.bottom_texture.clone() {
            textures.insert(BlockSide::Top, bottom_texture);
        }

        RenderShape::create_textured_cube(
            render_data,
            textured_block_type.side_texture.clone(),
            textures,
        );
 */
    }

    fn compile_colored_cube(&self) -> RenderShape {
        todo!()
    }
}

#[derive(Debug)]
pub struct TexturedCubeDesc {
    pub top_texture: Option<String>,
    pub side_texture: String,
    pub bottom_texture: Option<String>,
}

impl Default for TexturedCubeDesc {
    fn default() -> Self {
        Self {
            side_texture: "default".to_string(),
            top_texture: None,
            bottom_texture: None,
        }
    }
}
