use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

use crate::bevy_resources::BevyBlockTypeStorageAsset;

pub struct BlocksAssetPlugin;
impl Plugin for BlocksAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<BevyBlockTypeStorageAsset>::new(&[
            "blocks.json",
        ]))
        .init_asset::<BevyBlockTypeStorageAsset>();
    }
}
