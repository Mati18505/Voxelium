use bevy::prelude::*;
use thiserror::Error;

use crate::bevy_resources::{Dictionary, MaterialCompilationError, MaterialRuntimeSource, MaterialsDictionary, RuntimeMaterial};

#[derive(Debug)]
pub enum MaterialAsset {
    Placeholder,
    Textured { data: TexturedCubeMaterialData },
    Colored { data: ColoredCubeMaterialData },
    CutoutTextured { data: TexturedCubeMaterialData },
}

#[derive(Debug, Default)]
pub struct TexturedCubeMaterialData {
    pub texture_array_name: String,
}

#[derive(Debug, Default)]
pub struct ColoredCubeMaterialData {
    pub palette_name: String,
}

#[derive(Debug, Asset, TypePath)]
pub struct MaterialsDictAsset(pub MaterialsDictionary);

impl MaterialRuntimeSource for MaterialAsset {
    fn to_runtime(&self, textures_name_to_id: Dictionary<String, u32>) -> Result<RuntimeMaterial, MaterialCompilationError> {
        Ok(match self {
            MaterialAsset::Placeholder => RuntimeMaterial::Placeholder,
            MaterialAsset::Textured { data } => {
                let texture_name = &data.texture_array_name;
                let texture_id = textures_name_to_id.get(texture_name).ok_or(MaterialCompilationError::NoSuchTexture(texture_name.to_string()))?;

                RuntimeMaterial::Textured(*texture_id)
            }
            MaterialAsset::Colored { data } => {
                let texture_name = &data.palette_name;
                let texture_id = textures_name_to_id.get(texture_name).ok_or(MaterialCompilationError::NoSuchTexture(texture_name.to_string()))?;

                RuntimeMaterial::Colored(*texture_id)
            }
            MaterialAsset::CutoutTextured { data } => {
                let texture_name = &data.texture_array_name;
                let texture_id = textures_name_to_id.get(texture_name).ok_or(MaterialCompilationError::NoSuchTexture(texture_name.to_string()))?;

                RuntimeMaterial::CutoutTextured(*texture_id)
            }
        })
    }
}
