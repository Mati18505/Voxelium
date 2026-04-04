use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use serde::Deserialize;

use shared::entities::{BlockID, BlockType, BlockTypeStorage};

pub struct BlocksAssetPlugin;
impl Plugin for BlocksAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<BlockTypeStorageAsset>::new(&[
            "blocks.json",
        ]))
        .init_asset::<BlockTypeStorageAsset>();
    }
}

#[derive(Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
pub struct BlockTypeStorageAsset {
    blocks: Vec<BlockTypeAsset>,
}

pub type BlockNameToId = Vec<(String, BlockID)>;

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct BlockTypeAsset {
    name: String,
    affect_raycast: bool,
}

impl From<&BlockTypeAsset> for BlockType {
    fn from(v: &BlockTypeAsset) -> Self {
        BlockType {
            name: v.name.clone(),
            affect_raycast: v.affect_raycast,
        }
    }
}

impl From<&BlockTypeStorageAsset> for BlockTypeStorage {
    fn from(resource: &BlockTypeStorageAsset) -> Self {
        let block_types = resource
            .blocks
            .iter()
            .map(BlockType::from)
            .collect();

        BlockTypeStorage::new(block_types)
    }
}

impl From<&BlockTypeStorageAsset> for BlockNameToId {
    fn from(resource: &BlockTypeStorageAsset) -> Self {
        resource
            .blocks
            .iter()
            .enumerate()
            .map(|(i, e)| (e.name.clone(), i as BlockID))
            .collect()
    }
}
