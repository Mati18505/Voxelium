use bevy::log::warn;
use std::{collections::HashMap, default};

use shared::entities::{BlockSide, VoxelColor};

use crate::{
    bevy_resources::{texture_dictionary, MaterialName, TextureDictionary, TextureName},
    chunk_mesh_builder::{ColorIndex, RenderShape, TextureIndex, VoxelRenderData},
};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum RenderDesc {
    TexturedCube {
        render_data: RenderData,
        textured_cube_desc: TexturedCubeDesc,
    },
    ColoredCube {
        render_data: RenderData,
        color_index: ColorIndex,
    },
    Invisible,
}

impl RenderDesc {
    pub fn compile(&self, texture_dictionary: &TextureDictionary) -> RenderShape {
        match self {
            RenderDesc::TexturedCube {
                render_data,
                textured_cube_desc,
            } => Self::compile_textured_cube(render_data, textured_cube_desc, texture_dictionary),
            RenderDesc::ColoredCube {
                render_data,
                color_index,
            } => Self::compile_colored_cube(render_data, *color_index),
            RenderDesc::Invisible => RenderShape::Invisible,
        }
    }

    fn compile_textured_cube(
        rd: &RenderData,
        desc: &TexturedCubeDesc,
        texture_dictionary: &TextureDictionary,
    ) -> RenderShape {
        // TODO: get material from material dictionary
        let render_data = VoxelRenderData {
            visible: rd.visible,
            translucent: rd.translucent,
            material: 0,
        };

        let mut textures = HashMap::<BlockSide, TextureIndex>::default();

        if let Some(top_texture) = desc.top_texture.clone() {
            let index = Self::get_texture_index_from_name(&top_texture, texture_dictionary);
            textures.insert(BlockSide::Top, index);
        }
        if let Some(bottom_texture) = desc.bottom_texture.clone() {
            let index = Self::get_texture_index_from_name(&bottom_texture, texture_dictionary);
            textures.insert(BlockSide::Bottom, index);
        }

        let side_texture_index =
            Self::get_texture_index_from_name(&desc.side_texture, texture_dictionary);

        RenderShape::create_textured_cube(render_data, side_texture_index, textures)
    }

    fn get_texture_index_from_name(
        name: &str,
        texture_dictionary: &TextureDictionary,
    ) -> TextureIndex {
        match texture_dictionary.get_texture_index_from_name(&name) {
            Some(t_id) => t_id,
            None => {
                warn!("While compiling textured cube: texture \"{name}\" wasn't in texture_index.");
                0
            }
        }
    }

    fn compile_colored_cube(rd: &RenderData, color_index: ColorIndex) -> RenderShape {
        let render_data = VoxelRenderData {
            visible: rd.visible,
            translucent: rd.translucent,
            material: 1,
        };

        RenderShape::ColoredCube {
            render_data,
            color_index,
        }
    }
}

#[derive(Debug, Clone)]
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
