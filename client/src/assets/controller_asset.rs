use bevy::prelude::*;
use bevy_common_assets::yaml::YamlAssetPlugin;
use serde::Deserialize;

use bevy_flycam::MovementSettings;

pub struct ControllerAssetPlugin;
impl Plugin for ControllerAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(YamlAssetPlugin::<ControllerConfigAsset>::new(&[
            "controller.yaml",
        ]))
        .add_systems(Update, asset_changed);
    }
}

#[derive(Resource, Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
pub struct ControllerConfigAsset {
    pub sensitivity: f32,
    pub speed: f32,
}

impl From<&ControllerConfigAsset> for MovementSettings {
    fn from(v: &ControllerConfigAsset) -> Self {
        Self {
            sensitivity: v.sensitivity,
            speed: v.speed,
        }
    }
}

fn asset_changed(
    mut events: MessageReader<AssetEvent<ControllerConfigAsset>>,
    mut config: ResMut<MovementSettings>,
    assets: Res<Assets<ControllerConfigAsset>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } => {
                if let Some(asset) = assets.get(*id) {
                    info!("Movement settings changed: {:?}", asset);

                    let new_movement_settings: MovementSettings = asset.into();
                    *config = new_movement_settings;
                }
            }
            AssetEvent::Added { id } => {
                if let Some(asset) = assets.get(*id) {
                    info!("Movement settings loaded");

                    let new_movement_settings: MovementSettings = asset.into();
                    *config = new_movement_settings;
                }
            }
            _ => {}
        }
    }
}
