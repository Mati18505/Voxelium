use std::{collections::HashMap, path::PathBuf};

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    render::Render,
};
use thiserror::Error;

use crate::bevy_resources::{PaletteData, TextureArrayData, TextureAsset, TextureDictionary, TextureIndexDictionary, TextureName, texture_asset};

use yaml_rust2::{Yaml, YamlEmitter, YamlLoader, yaml::Hash};

#[derive(Debug, bevy::asset::Asset, bevy::reflect::TypePath)]
pub struct TextureDictAsset(pub TextureDictionary);

impl From<TextureDictAsset> for TextureDictionary {
    fn from(asset: TextureDictAsset) -> Self {
        asset.0
    }
}

#[derive(Default)]
pub struct TextureDictAssetLoader;

#[derive(Debug, Clone, Error)]
pub enum TextureDictAssetParseError {
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
}

#[derive(Debug, Error)]
pub enum TextureDictAssetLoaderError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not parse TextureDict: {0}")]
    Parse(#[from] TextureDictAssetParseError),
}

impl AssetLoader for TextureDictAssetLoader {
    fn extensions(&self) -> &[&str] {
        &["textures.yaml"]
    }

    type Asset = TextureDictAsset;
    type Settings = ();
    type Error = TextureDictAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        use TextureDictAssetParseError::*;

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let parsed = &YamlLoader::load_from_str(&String::from_utf8_lossy(&bytes)).map_err(|e| InvalidConfig(e.to_string()))?[0];

        if parsed["textures"].is_badvalue() {
            return Err(InvalidConfig("missing textures array".to_string()).into())
        }

        let textures_array: &Vec<Yaml> = parsed["textures"]
            .as_vec()
            .ok_or(InvalidConfig("textures should be an array".to_string()))?;

        let mut collected_textures = HashMap::default();

        for texture in textures_array {
            let texture: &Hash = texture.as_hash().ok_or(InvalidConfig("texture should be a hash".to_string()))?;

            for (k, v) in texture.iter() {
                let name: &str = k.as_str().ok_or(InvalidConfig("texture name should be a string".to_string()))?;
                let texture_type: &str = v["type"].as_str().ok_or(InvalidConfig("missing texture type".to_string()))?;
            
                let texture = match texture_type {
                    "array" => Ok(TextureAsset::TextureArray { data: parse_array_texture_params(&v)? }),
                    "palette" => Ok(TextureAsset::Palette { data: parse_palette_texture_params(&v)? }),
                    _ => Err(InvalidConfig(format!("unimplemented texture type {texture_type}"))),
                }?;

                collected_textures.insert(name.to_string(), texture);
            }
        }

        Ok(TextureDictAsset(TextureDictionary::new(collected_textures)))
    }
}

fn parse_array_texture_params(params: &Yaml) -> Result<TextureArrayData, TextureDictAssetLoaderError> {
    use TextureDictAssetParseError::*;

    let path: String = params["path"].as_str().ok_or(InvalidConfig("missing texture path".to_string()))?.to_string();
    let texture_indexes: &Hash = params["textures"].as_hash().ok_or(InvalidConfig("array texture must have defined texture indexes".to_string()))?;
    let mut textures = HashMap::new();

    for (k, v) in texture_indexes {
        let texture_name: &str = k.as_str().ok_or(InvalidConfig("texture name should be a string".to_string()))?;
        let texture_index: i64 = v.as_i64().ok_or(InvalidConfig("texture index should be an integer".to_string()))?;

        if texture_index < 0 {
            return Err(TextureDictAssetLoaderError::Parse(InvalidConfig("texture index should be a valid u32".to_string())));
        }

        let texture_index = texture_index as u32;

        textures.insert(texture_name.to_string(), texture_index);
    }

    Ok(TextureArrayData {
        path,
        textures: TextureIndexDictionary::new(textures), 
    })
}

fn parse_palette_texture_params(params: &Yaml) -> Result<PaletteData, TextureDictAssetLoaderError> {
    use TextureDictAssetParseError::*;

    let path: String = params["path"].as_str().ok_or(InvalidConfig("missing texture path".to_string()))?.to_string();
    let len: i64 = params["len"].as_i64().ok_or(InvalidConfig("palette texture must have defined palette length".to_string()))?;

    if len < 0 {
        return Err(TextureDictAssetLoaderError::Parse(InvalidConfig("len should be a valid u32".to_string())));
    }

    let len = len as usize;

    Ok(PaletteData {
        path,
        len,
    })
}