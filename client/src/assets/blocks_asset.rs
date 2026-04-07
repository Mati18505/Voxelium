use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use serde::Deserialize;

use shared::entities::{BlockID, BlockType, BlockTypeStorage};

use crate::bevy_resources::BlockNameToId;

pub struct BlocksAssetPlugin;
impl Plugin for BlocksAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<BlockTypeStorageAsset>::new(&[
            "blocks.json",
        ]))
        .init_asset::<BlockTypeStorageAsset>()
        .add_systems(Update, asset_changed);
    }
}

#[derive(Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
pub struct BlockTypeStorageAsset {
    blocks: Vec<BlockTypeAsset>,
}

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
        let block_types = resource.blocks.iter().map(BlockType::from).collect();

        BlockTypeStorage::new(block_types)
    }
}

impl From<&BlockTypeStorageAsset> for BlockNameToId {
    fn from(resource: &BlockTypeStorageAsset) -> Self {
        BlockNameToId::new(resource
            .blocks
            .iter()
            .enumerate()
            .map(|(i, e)| (e.name.clone(), i as BlockID))
            .collect())
    }
}

fn asset_changed(
    mut events: MessageReader<AssetEvent<BlockTypeStorageAsset>>,
    mut registry: ResMut<BlockNameToId>,
    assets: Res<Assets<BlockTypeStorageAsset>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } => {
                if let Some(asset) = assets.get(*id) {
                    info!("BlockTypeStorageAsset changed");

                    let new_block_storage: BlockNameToId = asset.into();
                    *registry = new_block_storage;
                }
            }
            AssetEvent::Added { id } => {
                if let Some(asset) = assets.get(*id) {
                    info!("BlockTypeStorageAsset loaded");

                    let new_block_storage: BlockNameToId = asset.into();
                    *registry = new_block_storage;
                }
            }
            _ => {}
        }
    }
}
