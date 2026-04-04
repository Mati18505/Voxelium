use bevy::prelude::*;

use crate::bevy_resources::{MaterialsDictAsset, MaterialsDictAssetLoader};

pub struct MaterialsDictAssetPlugin;
impl Plugin for MaterialsDictAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<MaterialsDictAssetLoader>()
            .init_asset::<MaterialsDictAsset>();
    }
}
