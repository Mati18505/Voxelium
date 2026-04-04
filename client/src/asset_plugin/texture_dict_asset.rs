use bevy::prelude::*;

pub use crate::asset_plugin::loaders::texture_dict_asset_loader::TextureDictAsset;
use crate::asset_plugin::loaders::texture_dict_asset_loader::TextureDictAssetLoader;

pub struct TextureDictAssetPlugin;
impl Plugin for TextureDictAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<TextureDictAssetLoader>()
            .init_asset::<TextureDictAsset>();
    }
}
