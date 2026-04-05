use bevy::prelude::*;

use super::{material_asset_loader::MaterialsDictAssetLoader, MaterialsDictAsset};

pub struct MaterialsDictAssetPlugin;
impl Plugin for MaterialsDictAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<MaterialsDictAssetLoader>()
            .init_asset::<MaterialsDictAsset>();
    }
}
