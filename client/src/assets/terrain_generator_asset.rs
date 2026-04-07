use bevy::prelude::*;
use bevy_common_assets::yaml::YamlAssetPlugin;
use serde::Deserialize;
use shared::chunk_io::providers::terrain_generator::TerrainConfig;

use crate::chunk_manager::TerrainGeneratorConfig;

pub struct TerrainGeneratorAssetPlugin;
impl Plugin for TerrainGeneratorAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(YamlAssetPlugin::<TerrainGeneratorConfigAsset>::new(&[
            "terrain_generator.yaml",
        ]))
        .add_systems(Update, asset_changed);
    }
}

#[derive(Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
pub struct TerrainGeneratorConfigAsset {
    pub seed: i32,
    pub freq: f32,
    pub lacunarity: f32,
    pub octaves: u8,
    pub add_flat_noise: bool,
}

impl From<&TerrainGeneratorConfigAsset> for TerrainConfig {
    fn from(v: &TerrainGeneratorConfigAsset) -> Self {
        Self {
            seed: v.seed,
            freq: v.freq,
            lacunarity: v.lacunarity,
            octaves: v.octaves,
            add_flat_noise: v.add_flat_noise,
        }
    }
}

fn asset_changed(
    mut events: MessageReader<AssetEvent<TerrainGeneratorConfigAsset>>,
    mut config: ResMut<TerrainGeneratorConfig>,
    assets: Res<Assets<TerrainGeneratorConfigAsset>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } => {
                if let Some(asset) = assets.get(*id) {
                    info!("TerrainGeneratorConfig changed: {:?}", asset);

                    let new_config: TerrainConfig = asset.into();
                    config.0 = new_config;
                }
            }
            AssetEvent::Added { id } => {
                if let Some(asset) = assets.get(*id) {
                    info!("TerrainGeneratorConfig loaded");

                    let new_config: TerrainConfig = asset.into();
                    config.0 = new_config;
                }
            }
            _ => {}
        }
    }
}
