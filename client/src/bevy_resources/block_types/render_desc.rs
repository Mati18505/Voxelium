use bevy::log::warn;
use std::collections::HashMap;
use thiserror::Error;

use shared::entities::BlockSide;

use crate::{
    bevy_resources::{
        Dictionary, MaterialName,
        TextureAsset::{Palette, TextureArray},
        TextureDictionary, TextureIndexDictionary, TextureName,
    },
    chunk_mesh_builder::{ColorIndex, MaterialId, RenderShape, TextureIndex, VoxelRenderData},
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

#[derive(Debug, Clone, Error)]
pub enum RenderDescCompilationError {
    #[error("No such material: {0}")]
    NoSuchMaterial(MaterialName),
    #[error("Texture asset was not found: {0}")]
    NoTextureAsset(TextureName),
    #[error("Material {0} is invalid for use in: {1}")]
    InvalidMaterial(MaterialName, String),
}

impl RenderDesc {
    pub fn compile(
        &self,
        material_name_to_id: &Dictionary<MaterialName, MaterialId>,
        material_id_to_texture_name: &Dictionary<MaterialId, TextureName>,
        texture_asset_dictionary: &TextureDictionary,
    ) -> Result<RenderShape, RenderDescCompilationError> {
        match self {
            RenderDesc::TexturedCube {
                render_data,
                textured_cube_desc,
            } => Self::compile_textured_cube(
                render_data,
                textured_cube_desc,
                material_name_to_id,
                material_id_to_texture_name,
                texture_asset_dictionary,
            ),
            RenderDesc::ColoredCube {
                render_data,
                color_index,
            } => Self::compile_colored_cube(render_data, *color_index, material_name_to_id),
            RenderDesc::Invisible => Ok(RenderShape::Invisible),
        }
    }

    fn compile_textured_cube(
        rd: &RenderData,
        desc: &TexturedCubeDesc,
        material_name_to_id: &Dictionary<MaterialName, MaterialId>,
        material_id_to_texture_name: &Dictionary<MaterialId, TextureName>,
        texture_asset_dictionary: &TextureDictionary,
    ) -> Result<RenderShape, RenderDescCompilationError> {
        use RenderDescCompilationError::*;

        let material = *material_name_to_id
            .get(&rd.material)
            .ok_or(NoSuchMaterial(rd.material.to_string()))?;

        let texture_name = material_id_to_texture_name
            .get(&material)
            .ok_or(InvalidMaterial(
                rd.material.to_string(),
                "textured_cube".to_string(),
            ))?;
        let texture_asset = texture_asset_dictionary
            .get(&texture_name)
            .ok_or(NoTextureAsset(texture_name.to_string()))?;

        let texture_dictionary = match texture_asset {
            TextureArray { data } => &data.textures,
            Palette { .. } => unreachable!(),
        };

        let render_data = VoxelRenderData {
            visible: rd.visible,
            translucent: rd.translucent,
            material,
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

        Ok(RenderShape::create_textured_cube(
            render_data,
            side_texture_index,
            textures,
        ))
    }

    fn get_texture_index_from_name(
        name: &str,
        texture_dictionary: &TextureIndexDictionary,
    ) -> TextureIndex {
        match texture_dictionary.get(&name.to_string()) {
            Some(t_id) => *t_id,
            None => {
                warn!("While compiling textured cube: texture \"{name}\" wasn't in texture_index.");
                0
            }
        }
    }

    fn compile_colored_cube(
        rd: &RenderData,
        color_index: ColorIndex,
        material_name_to_id: &Dictionary<MaterialName, MaterialId>,
    ) -> Result<RenderShape, RenderDescCompilationError> {
        use RenderDescCompilationError::*;

        let material = *material_name_to_id
            .get(&rd.material)
            .ok_or(NoSuchMaterial(rd.material.to_string()))?;

        let render_data = VoxelRenderData {
            visible: rd.visible,
            translucent: rd.translucent,
            material,
        };

        Ok(RenderShape::ColoredCube {
            render_data,
            color_index,
        })
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

#[cfg(test)]
mod tests {
    use crate::bevy_resources::{PaletteData, TextureArrayData};

    use super::*;

    #[test]
    fn test_compile_textured_cube_compiles_successfully() {
        let render_desc = RenderDesc::TexturedCube {
            render_data: RenderData {
                material: "stone_material".to_string(),
                ..Default::default()
            },
            textured_cube_desc: TexturedCubeDesc {
                side_texture: "stone".to_string(),
                top_texture: Some("stone_top".to_string()),
                bottom_texture: Some("stone_bottom".to_string()),
            },
        };

        let mut material_name_to_id = Dictionary::<MaterialName, MaterialId>::default();
        material_name_to_id.set("stone_material".to_string(), 0);

        let mut material_id_to_texture_name = Dictionary::<MaterialId, TextureName>::default();
        material_id_to_texture_name.set(0, "block_textures".to_string());

        let mut texture_indices = TextureIndexDictionary::default();
        texture_indices.set("stone".to_string(), 1);
        texture_indices.set("stone_top".to_string(), 2);
        texture_indices.set("stone_bottom".to_string(), 3);

        let mut texture_asset_dictionary = TextureDictionary::default();
        texture_asset_dictionary.set(
            "block_textures".to_string(),
            TextureArray {
                data: TextureArrayData {
                    path: "opaque.png".to_string(),
                    textures: texture_indices,
                },
            },
        );

        let result = render_desc.compile(
            &material_name_to_id,
            &material_id_to_texture_name,
            &texture_asset_dictionary,
        );

        assert!(matches!(result, Ok(RenderShape::TexturedCube { .. })));
    }

    #[test]
    fn test_compile_textured_cube_invalid_material() {
        let render_desc = RenderDesc::TexturedCube {
            render_data: RenderData {
                material: "stone_material".to_string(),
                ..Default::default()
            },
            textured_cube_desc: TexturedCubeDesc {
                side_texture: "stone".to_string(),
                top_texture: Some("stone_top".to_string()),
                bottom_texture: Some("stone_bottom".to_string()),
            },
        };

        let mut material_name_to_id = Dictionary::<MaterialName, MaterialId>::default();
        material_name_to_id.set("stone_material".to_string(), 0);

        let mut material_id_to_texture_name = Dictionary::<MaterialId, TextureName>::default();

        let mut texture_indices = TextureIndexDictionary::default();
        texture_indices.set("stone".to_string(), 1);
        texture_indices.set("stone_top".to_string(), 2);
        texture_indices.set("stone_bottom".to_string(), 3);

        let mut texture_asset_dictionary = TextureDictionary::default();
        texture_asset_dictionary.set(
            "block_textures".to_string(),
            TextureArray {
                data: TextureArrayData {
                    path: "opaque.png".to_string(),
                    textures: texture_indices,
                },
            },
        );

        let result = render_desc.compile(
            &material_name_to_id,
            &material_id_to_texture_name,
            &texture_asset_dictionary,
        );

        assert!(matches!(
            result,
            Err(RenderDescCompilationError::InvalidMaterial(..))
        ));
    }
}
