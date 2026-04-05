use bevy::prelude::*;
use bevy_common_assets::yaml::YamlAssetPlugin;
use serde::Deserialize;

use crate::chunk_manager::ChunkBuilderConfig;

pub struct ChunkBuilderAssetPlugin;
impl Plugin for ChunkBuilderAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(YamlAssetPlugin::<ChunkBuilderConfigAsset>::new(&[
            "chunk_builder.yaml",
        ]))
        .add_systems(Update, asset_changed);
    }
}

#[derive(Resource, Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
pub struct ChunkBuilderConfigAsset {
    pub max_builds_per_frame: usize,
    pub render_distance: usize,
    pub dynamic_vertical_loading: bool,
    pub debug: bool,
}

impl From<&ChunkBuilderConfigAsset> for ChunkBuilderConfig {
    fn from(value: &ChunkBuilderConfigAsset) -> Self {
        Self {
            max_builds_per_frame: value.max_builds_per_frame,
            render_distance: value.render_distance,
            dynamic_vertical_loading: value.dynamic_vertical_loading,
            debug: value.debug,
        }
    }
}

fn asset_changed(
    mut events: MessageReader<AssetEvent<ChunkBuilderConfigAsset>>,
    mut config: ResMut<ChunkBuilderConfig>,
    assets: Res<Assets<ChunkBuilderConfigAsset>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } => {
                if let Some(asset) = assets.get(*id) {
                    info!("ChunkBuilderConfig changed: {:?}", asset);

                    let new_chunk_builder_config: ChunkBuilderConfig = asset.into();
                    *config = new_chunk_builder_config;
                }
            }
            AssetEvent::Added { id } => {
                if let Some(asset) = assets.get(*id) {
                    info!("ChunkBuilderConfig loaded");

                    let new_chunk_builder_config: ChunkBuilderConfig = asset.into();
                    *config = new_chunk_builder_config;
                }
            }
            _ => {}
        }
    }
}
