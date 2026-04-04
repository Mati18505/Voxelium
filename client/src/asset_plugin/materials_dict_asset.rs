use bevy::prelude::*;

pub use super::loaders::materials_dict_asset_loader::MaterialsDictAsset;

use crate::asset_plugin::loaders::materials_dict_asset_loader::MaterialsDictAssetLoader;

pub struct MaterialsDictAssetPlugin;
impl Plugin for MaterialsDictAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<MaterialsDictAssetLoader>()
            .init_asset::<MaterialsDictAsset>();
    }
}
