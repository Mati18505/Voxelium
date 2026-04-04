use bevy::prelude::*;

use crate::bevy_resources::{RenderDescDictAsset, RenderDescDictAssetLoader};

pub struct RenderDescAssetPlugin;
impl Plugin for RenderDescAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<RenderDescDictAssetLoader>()
            .init_asset::<RenderDescDictAsset>();
    }
}
