use std::collections::HashMap;

use shared::entities::{BlockID, BlockSide};
use thiserror::Error;

use bevy::{asset::{AssetLoader, LoadContext, io::Reader}, render::Render};

use crate::bevy_resources::{RenderBlockType, RenderBlockTypeStorage, RenderData, RenderDesc, TexturedBlockTypeBuilder};

#[derive(bevy::asset::Asset, bevy::reflect::TypePath, Debug, Clone)]
pub struct RenderBlockTypeStorageResource {
    block_types: Vec<RenderBlockType>,
}

impl From<RenderBlockTypeStorageResource> for RenderBlockTypeStorage {
    fn from(resource: RenderBlockTypeStorageResource) -> Self {
        RenderBlockTypeStorage::new(resource.block_types)
    }
}

#[derive(Default)]
pub struct RenderBlockTypeStorageLoader;

#[derive(Debug, Clone, Error)]
pub enum BlockStorageParseError {
    #[error("Invalid config: {0}")]
    InvalidConfig(String),
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
            .ok_or(InvalidConfig("missing blocks array".to_string()))?
            .as_array()
            .ok_or(InvalidConfig("blocks should be array".to_string()))?;

        let mut block_type_storage = RenderBlockTypeStorageResource {
            block_types: Vec::new(),
        };

        for block in blocks {
            let block_type: String = match block.get("block_type") {
                Some(v) => v
                    .as_str()
                    .ok_or(InvalidConfig("block_type parameter should be string".to_string()))?,
                None => return Err(InvalidConfig("missing block_type parameter".to_string()).into()),
            }
            .to_owned();
            let material_name: String = match block.get("material") {
                Some(v) => v
                    .as_str()
                    .ok_or(InvalidConfig("material parameter should be string".to_string()))?,
                None => "default",
            }
            .to_owned();
            let visible: bool = match block.get("visible") {
                Some(v) => v
                    .as_bool()
                    .ok_or(InvalidConfig("visible parameter should be boolean".to_string()))?,
                None => true,
            };
            let translucent: bool = match block.get("translucent") {
                Some(v) => v
                    .as_bool()
                    .ok_or(InvalidConfig("translucent paramterer should be boolean".to_string()))?,
                None => !visible,
            };

            let render_data = RenderData {
                visible,
                translucent,
                material: material_name.clone(),
            };

            let render_block_type: RenderBlockType = match material_name.as_str() {
                "default" => {
                    let mut builder = TexturedBlockTypeBuilder::new(&block_type)
                        .render_data(render_data);

                    if let Some(textures) = block.get("textures") {
                        builder = add_textures(builder, textures);
                    }

                    Ok(builder.build())
                },
                "colored" => {
                    let palette_index: u64 = match block.get("palette_index") {
                        Some(v) => v
                            .as_u64()
                            .ok_or(InvalidConfig("palette_index parameter should be unsigned 32bit number".to_string()))?,
                        None => 0,
                    };

                    Ok(RenderBlockType {
                        block_type,
                        render_desc: RenderDesc::ColoredCube { render_data, palette_index: palette_index as u32 }
                    })
                }

                _ => Err(InvalidConfig(format!("unsupported material {material_name}"))),
            }?;
            
            block_type_storage.block_types.push(render_block_type);
        }

        Ok(block_type_storage)
    }
}

fn add_textures(builder: TexturedBlockTypeBuilder, textures: &serde_json::Value) -> TexturedBlockTypeBuilder {
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