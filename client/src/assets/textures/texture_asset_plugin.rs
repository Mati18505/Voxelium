use bevy::prelude::*;

use super::{texture_asset_loader::TextureDictAssetLoader, TextureDictAsset};

pub struct TextureAssetPlugin;
impl Plugin for TextureAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<TextureDictAssetLoader>()
            .init_asset::<TextureDictAsset>();
    }
}
