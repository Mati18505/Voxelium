use std::collections::HashMap;

use shared::entities::{BlockID, BlockSide};
use thiserror::Error;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    render::Render,
};

use crate::bevy_resources::{
    BlockTypeName, ColoredCubeMaterialData, MaterialAsset, MaterialName, MaterialsDictionary,
    TexturedBlockTypeBuilder, TexturedCubeMaterialData,
};

#[derive(Debug, bevy::asset::Asset, bevy::reflect::TypePath)]
pub struct MaterialsDictAsset(pub MaterialsDictionary);

#[derive(Default)]
pub struct MaterialsDictAssetLoader;

#[derive(Debug, Clone, Error)]
pub enum MaterialsDictAssetParseError {
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Error)]
pub enum MaterialsDictAssetLoaderError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Could not parse MaterialsDict: {0}")]
    Parse(#[from] MaterialsDictAssetParseError),
}

impl AssetLoader for MaterialsDictAssetLoader {
    fn extensions(&self) -> &[&str] {
        &["materials.json"]
    }

    type Asset = MaterialsDictAsset;
    type Settings = ();
    type Error = MaterialsDictAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        use MaterialsDictAssetParseError::*;

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let parsed: serde_json::Value = serde_json::from_slice(&bytes)?;
        let materials_data: &Vec<serde_json::Value> = parsed
            .get("materials")
            .ok_or(InvalidConfig("missing materials array".to_string()))?
            .as_array()
            .ok_or(InvalidConfig("materials should be array".to_string()))?;

        let mut materials: HashMap<MaterialName, MaterialAsset> = HashMap::default();

        for material in materials_data {
            let material_name: String = match material.get("name") {
                Some(v) => v
                    .as_str()
                    .ok_or(InvalidConfig("name parameter should be string".to_string()))?,
                None => return Err(InvalidConfig("missing name parameter".to_string()).into()),
            }
            .to_owned();
            let material_type: String = match material.get("type") {
                Some(v) => v
                    .as_str()
                    .ok_or(InvalidConfig("type parameter should be string".to_string()))?,
                None => return Err(InvalidConfig("missing type parameter".to_string()).into()),
            }
            .to_owned();

            let mut material_asset: MaterialAsset = match material_type.as_str() {
                "textured_cube" => Ok(MaterialAsset::TexturedCube {
                    data: TexturedCubeMaterialData::default(),
                }),
                "colored_cube" => Ok(MaterialAsset::ColoredCube {
                    data: ColoredCubeMaterialData::default(),
                }),
                "cutout_textured_cube" => Ok(MaterialAsset::CutoutTexturedCube {
                    data: TexturedCubeMaterialData::default(),
                }),
                _ => Err(InvalidConfig(format!(
                    "unsupported material type: {material_type}"
                ))),
            }?;

            match material_asset {
                MaterialAsset::TexturedCube { ref mut data } => {
                    *data = load_textured_cube_data(material)?;
                }
                MaterialAsset::ColoredCube { ref mut data } => {
                    *data = load_colored_cube_data(material)?;
                }
                MaterialAsset::CutoutTexturedCube { ref mut data } => {
                    *data = load_textured_cube_data(material)?;
                }
            }

            materials.insert(material_name, material_asset);
        }

        let materials_dict = MaterialsDictionary::new(materials);

        Ok(MaterialsDictAsset(materials_dict))
    }
}

fn load_textured_cube_data(
    material: &serde_json::Value,
) -> Result<TexturedCubeMaterialData, MaterialsDictAssetLoaderError> {
    use MaterialsDictAssetParseError::*;
    let texture_array_name: String = match material.get("texture_array") {
        Some(v) => v.as_str().ok_or(InvalidConfig(
            "texture_array parameter should be string".to_string(),
        ))?,
        None => return Err(InvalidConfig("missing texture_array parameter".to_string()).into()),
    }
    .to_owned();

    Ok(TexturedCubeMaterialData { texture_array_name })
}

fn load_colored_cube_data(
    material: &serde_json::Value,
) -> Result<ColoredCubeMaterialData, MaterialsDictAssetLoaderError> {
    use MaterialsDictAssetParseError::*;
    let palette_name: String = match material.get("palette") {
        Some(v) => v.as_str().ok_or(InvalidConfig(
            "palette parameter should be string".to_string(),
        ))?,
        None => return Err(InvalidConfig("missing palette parameter".to_string()).into()),
    }
    .to_owned();

    Ok(ColoredCubeMaterialData { palette_name })
}

fn add_textures(
    builder: TexturedBlockTypeBuilder,
    textures: &serde_json::Value,
) -> TexturedBlockTypeBuilder {
    let mut builder = builder;

    if let Some(side_texture) = textures.get("side").and_then(|e| e.as_str()) {
        builder = builder.texture(BlockSide::Left, side_texture);
    }
    if let Some(top_texture) = textures.get("top").and_then(|e| e.as_str()) {
        builder = builder.texture(BlockSide::Top, top_texture);
    }
    if let Some(bottom_texture) = textures.get("bottom").and_then(|e| e.as_str()) {
        builder = builder.texture(BlockSide::Bottom, bottom_texture);
    }

    builder
}
