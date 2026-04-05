use bevy::prelude::*;
use bevy_common_assets::yaml::YamlAssetPlugin;
use serde::Deserialize;

use crate::chunk_manager::ChunkLoaderConfig;

pub struct ChunkLoaderAssetPlugin;
impl Plugin for ChunkLoaderAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(YamlAssetPlugin::<ChunkLoaderConfigAsset>::new(&[
            "chunk_loader.yaml",
        ]))
        .add_systems(Update, asset_changed);
    }
}

#[derive(Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
pub struct ChunkLoaderConfigAsset {
    pub max_loads_per_frame: usize,
    pub load_distance: usize,
    pub dynamic_vertical_loading: bool,
    pub debug: bool,
}

impl From<&ChunkLoaderConfigAsset> for ChunkLoaderConfig {
    fn from(value: &ChunkLoaderConfigAsset) -> Self {
        Self {
            max_loads_per_frame: value.max_loads_per_frame,
            load_distance: value.load_distance,
            dynamic_vertical_loading: value.dynamic_vertical_loading,
            debug: value.debug,
        }
    }
}

fn asset_changed(
    mut events: MessageReader<AssetEvent<ChunkLoaderConfigAsset>>,
    mut config: ResMut<ChunkLoaderConfig>,
    assets: Res<Assets<ChunkLoaderConfigAsset>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } => {
                if let Some(asset) = assets.get(*id) {
                    info!("ChunkLoaderConfig changed: {:?}", asset);

                    let new_chunk_loader_config: ChunkLoaderConfig = asset.into();
                    *config = new_chunk_loader_config;
                }
            }
            AssetEvent::Added { id } => {
                if let Some(asset) = assets.get(*id) {
                    info!("ChunkLoaderConfig loaded");

                    let new_chunk_loader_config: ChunkLoaderConfig = asset.into();
                    *config = new_chunk_loader_config;
                }
            }
            _ => {}
        }
    }
}
