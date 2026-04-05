use bevy::prelude::*;

use super::{render_desc_asset_loader::RenderDescDictAssetLoader, RenderDescDictAsset};

pub struct RenderDescAssetPlugin;
impl Plugin for RenderDescAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<RenderDescDictAssetLoader>()
            .init_asset::<RenderDescDictAsset>();
    }
}
