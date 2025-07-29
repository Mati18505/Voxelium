use std::collections::HashMap;

use shared::entities::{BlockID, BlockSide};
use thiserror::Error;

use bevy::asset::{io::Reader, AssetLoader, LoadContext};

use crate::chunk_mesh_builder::{TexturedBlockType, TexturedBlockTypeBuilder, RenderBlockTypeStorage};

#[derive(bevy::asset::Asset, bevy::reflect::TypePath, Debug, Clone, PartialEq)]
pub struct RenderBlockTypeStorageResource {
    block_types: Vec<TexturedBlockType>,
}

impl From<RenderBlockTypeStorageResource> for RenderBlockTypeStorage {
    fn from(resource: RenderBlockTypeStorageResource) -> Self {
        let block_types: HashMap<BlockID, TexturedBlockType> = resource
            .block_types
            .into_iter()
            .enumerate()
            .map(|(i, e)| (i as u8, e))
            .collect();

        RenderBlockTypeStorage::new(block_types)
    }
}

#[derive(Default)]
pub struct RenderBlockTypeStorageLoader;

#[derive(Debug, Clone, Error)]
pub enum BlockStorageParseError {
    #[error("Invalid config: {0}")]
    InvalidConfig(&'static str),
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

impl AssetLoader for RenderBlockTypeStorageLoader {
    fn extensions(&self) -> &[&str] {
        &["blocks.json"]
    }

    type Asset = RenderBlockTypeStorageResource;
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
            .ok_or(InvalidConfig("missing blocks array"))?
            .as_array()
            .ok_or(InvalidConfig("blocks should be array"))?;

        let mut block_type_storage = RenderBlockTypeStorageResource {
            block_types: Vec::new(),
        };

        for block in blocks {
            let block_type: String = match block.get("block_type") {
                Some(v) => v
                    .as_str()
                    .ok_or(InvalidConfig("block_type parameter should be string"))?,
                None => return Err(InvalidConfig("missing block_type parameter").into()),
            }
            .to_owned();
            let material_name: String = match block.get("material") {
                Some(v) => v
                    .as_str()
                    .ok_or(InvalidConfig("material parameter should be string"))?,
                None => "default",
            }
            .to_owned();
            let visible: bool = match block.get("visible") {
                Some(v) => v
                    .as_bool()
                    .ok_or(InvalidConfig("visible parameter should be boolean"))?,
                None => true,
            };
            let translucent: bool = match block.get("translucent") {
                Some(v) => v
                    .as_bool()
                    .ok_or(InvalidConfig("translucent paramterer should be boolean"))?,
                None => !visible,
            };

            let mut builder = TexturedBlockTypeBuilder::new(&block_type)
                .visible(visible)
                .material(material_name)
                .translucent(translucent);

            if let Some(textures) = block.get("textures") {
                if let Some(side_texture) = textures.get("side").and_then(|e| e.as_str()) {
                    builder = builder.texture(BlockSide::Left, side_texture);
                }
                if let Some(top_texture) = textures.get("top").and_then(|e| e.as_str()) {
                    builder = builder.texture(BlockSide::Top, top_texture);
                }
                if let Some(bottom_texture) = textures.get("bottom").and_then(|e| e.as_str()) {
                    builder = builder.texture(BlockSide::Bottom, bottom_texture);
                }
            }

            block_type_storage.block_types.push(builder.build());
        }
        Ok(block_type_storage)
    }
}
