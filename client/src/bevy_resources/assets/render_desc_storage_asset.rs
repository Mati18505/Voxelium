use std::collections::HashMap;

use bevy::reflect::TypePath;
use shared::entities::BlockSide;
use thiserror::Error;

use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::log::warn;

use crate::bevy_resources::{
    BlockTypeName, RenderData, RenderDesc, RenderDescDictionary, TexturedBlockTypeBuilder,
};

#[derive(Debug, bevy::asset::Asset, bevy::reflect::TypePath, Clone)]
pub struct RenderDescDictAsset(pub RenderDescDictionary);

#[derive(Default, TypePath)]
pub struct RenderDescDictAssetLoader;

#[derive(Debug, Clone, Error)]
pub enum BlockStorageParseError {
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
    #[error("Invalid entry \"{0}\": {1}")]
    InvalidEntry(BlockTypeName, String),
}

#[derive(Debug, Error)]
pub enum BlockStorageLoaderError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Could not parse BlockStorage: {0}")]
    Parse(#[from] BlockStorageParseError),
}

impl AssetLoader for RenderDescDictAssetLoader {
    fn extensions(&self) -> &[&str] {
        &["render_desc.json"]
    }

    type Asset = RenderDescDictAsset;
    type Settings = ();
    type Error = BlockStorageLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        use BlockStorageParseError::*;

        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let parsed: serde_json::Value = serde_json::from_slice(&bytes)?;
        let blocks: &Vec<serde_json::Value> = parsed
            .get("blocks")
            .ok_or(InvalidConfig("missing blocks array".to_string()))?
            .as_array()
            .ok_or(InvalidConfig("blocks should be array".to_string()))?;

        let mut render_descriptions: HashMap<BlockTypeName, RenderDesc> = HashMap::default();

        for block in blocks {
            match parse_single_block(block) {
                Ok((block_type_name, render_desc)) => {
                    render_descriptions.insert(block_type_name, render_desc);
                }
                Err(err) => {
                    warn!("Skipping render_desc entry: {err}");
                }
            }
        }

        let render_desc_dict = RenderDescDictionary::new(render_descriptions);

        Ok(RenderDescDictAsset(render_desc_dict))
    }
}

fn parse_single_block(
    block: &serde_json::Value,
) -> Result<(BlockTypeName, RenderDesc), BlockStorageParseError> {
    use BlockStorageParseError::*;

    let block_type_name: String = match block.get("block_type") {
        Some(v) => v.as_str().ok_or(InvalidConfig(
            "block_type parameter should be string".to_string(),
        ))?,
        None => return Err(InvalidConfig("missing block_type parameter".to_string())),
    }
    .to_owned();
    let type_name: String = match block.get("type") {
        Some(v) => v.as_str().ok_or(InvalidEntry(
            block_type_name.clone(),
            "type parameter should be string".to_string(),
        ))?,
        None => "textured",
    }
    .to_owned();
    let material_name: String = match block.get("material") {
        Some(v) => v.as_str().ok_or(InvalidEntry(
            block_type_name.clone(),
            "material parameter should be string".to_string(),
        ))?,
        None => "default",
    }
    .to_owned();
    let visible: bool = match block.get("visible") {
        Some(v) => v.as_bool().ok_or(InvalidEntry(
            block_type_name.clone(),
            "visible parameter should be boolean".to_string(),
        ))?,
        None => true,
    };
    let translucent: bool = match block.get("translucent") {
        Some(v) => v.as_bool().ok_or(InvalidEntry(
            block_type_name.clone(),
            "translucent paramterer should be boolean".to_string(),
        ))?,
        None => !visible,
    };

    let render_data = RenderData {
        visible,
        translucent,
        material: material_name,
    };

    if !visible {
        return Ok((block_type_name, RenderDesc::Invisible));
    }

    let render_desc: RenderDesc = match type_name.as_str() {
        "textured" => {
            let mut builder = TexturedBlockTypeBuilder::new().render_data(render_data);

            let textures = block.get("textures").ok_or(InvalidEntry(
                block_type_name.clone(),
                "default block_type should have textures".to_string(),
            ))?;
            builder = add_textures(builder, textures);

            Ok(builder.build())
        }
        "colored" => {
            let color_index = block.get("color_index").ok_or(InvalidEntry(
                block_type_name.clone(),
                "colored block_type should have color_index".to_string(),
            ))?;
            let color_index: u64 = color_index.as_u64().ok_or(InvalidEntry(
                block_type_name.clone(),
                "color_index should be unsigned 32bit number".to_string(),
            ))?;

            let render_desc = RenderDesc::ColoredCube {
                render_data,
                color_index: color_index as u32,
            };

            Ok(render_desc)
        }
        _ => Err(InvalidEntry(
            block_type_name.clone(),
            format!("unsupported type {type_name}"),
        )),
    }?;

    Ok((block_type_name, render_desc))
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
