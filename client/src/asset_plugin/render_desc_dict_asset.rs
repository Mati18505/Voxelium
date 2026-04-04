use bevy::prelude::*;

pub use super::loaders::render_desc_storage_asset::RenderDescDictAsset;
use super::loaders::render_desc_storage_asset::RenderDescDictAssetLoader;

pub struct RenderDescAssetPlugin;
impl Plugin for RenderDescAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<RenderDescDictAssetLoader>()
            .init_asset::<RenderDescDictAsset>();
    }
}
