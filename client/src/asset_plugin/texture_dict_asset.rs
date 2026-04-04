use bevy::prelude::*;

use crate::bevy_resources::{TextureDictAsset, TextureDictAssetLoader};

pub struct TextureDictAssetPlugin;
impl Plugin for TextureDictAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<TextureDictAssetLoader>()
            .init_asset::<TextureDictAsset>();
    }
}
