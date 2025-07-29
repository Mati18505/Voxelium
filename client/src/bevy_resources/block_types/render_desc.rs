use std::{collections::HashMap, default};

use bevy::render::render_resource::Texture;
use shared::entities::{BlockSide, VoxelColor};

use crate::{
    bevy_resources::{texture_dictionary, MaterialName, TextureDictionary, TextureName},
    chunk_mesh_builder::{RenderShape, VoxelRenderData},
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
    pub fn compile(&self, texture_dictionary: &TextureDictionary) -> RenderShape {
        match self {
            RenderDesc::TexturedCube(rd, desc) => {
                self.compile_textured_cube(rd, desc, texture_dictionary)
            }
            RenderDesc::ColoredCube(rd, palette) => self.compile_colored_cube(rd, palette),
            RenderDesc::Invisible => RenderShape::Invisible,
        }
    }

    fn compile_textured_cube(
        rd: RenderData,
        desc: TexturedCubeDesc,
        texture_dictionary: &TextureDictionary,
    ) -> RenderShape {
        // TODO: get material from material dictionary
        let render_data = VoxelRenderData {
            visible: rd.visible,
            material: 0,
        };

        let mut textures = HashMap::<BlockSide, TextureName>::default();

        if let Some(top_texture) = desc.top_texture.clone() {
            textures.insert(BlockSide::Top, top_texture);
        }
        if let Some(bottom_texture) = desc.bottom_texture.clone() {
            textures.insert(BlockSide::Bottom, bottom_texture);
        }

        RenderShape::create_textured_cube(render_data, desc.side_texture, textures)
    }

    fn compile_colored_cube(rd: RenderData, palette: Vec<VoxelColor>) -> RenderShape {
        let render_data = VoxelRenderData {
            visible: rd.visible,
            material: 0,
        };

        RenderShape::ColoredCube {
            render_data,
            palette,
        }
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
