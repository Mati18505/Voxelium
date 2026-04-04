use bevy::prelude::*;

pub use super::loaders::render_desc_storage_asset_loader::RenderDescDictAsset;
use super::loaders::render_desc_storage_asset_loader::RenderDescDictAssetLoader;

pub struct RenderDescAssetPlugin;
impl Plugin for RenderDescAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<RenderDescDictAssetLoader>()
            .init_asset::<RenderDescDictAsset>();
    }
}
