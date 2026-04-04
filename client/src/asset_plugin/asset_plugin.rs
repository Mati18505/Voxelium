use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use super::blocks_asset::BlocksAssetPlugin;
use super::chunk_loader_asset::{ChunkLoaderAssetPlugin, ChunkLoaderConfigAsset};
use super::materials_dict_asset::MaterialsDictAssetPlugin;
use super::render_desc_dict_asset::RenderDescAssetPlugin;
use super::texture_dict_asset::TextureDictAssetPlugin;

use crate::{
    bevy_resources::{
        BevyBlockTypeStorageAsset, MaterialsDictAsset, RenderDescDictAsset, TextureDictAsset,
    },
    bevy_types::AppStates,
};

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChunkLoaderAssetPlugin,
            BlocksAssetPlugin,
            RenderDescAssetPlugin,
            MaterialsDictAssetPlugin,
            TextureDictAssetPlugin,
        ))
        .add_loading_state(
            LoadingState::new(AppStates::Loading)
                .continue_to_state(AppStates::Compile)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "texture_array.assets.ron",
                )
                .load_collection::<VoxelAssets>()
                .load_collection::<Config>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct Config {
    #[asset(path = "config.chunk_loader.yaml")]
    pub chunk_loader_settings: Handle<ChunkLoaderConfigAsset>,
}

#[derive(AssetCollection, Resource)]
pub struct VoxelAssets {
    #[asset(path = "global.render_desc.json")]
    pub render_desc_storage_res: Handle<RenderDescDictAsset>,
    #[asset(path = "global.textures.yaml")]
    pub texture_dict_asset: Handle<TextureDictAsset>,
    #[asset(path = "global.blocks.json")]
    pub server_blocks: Handle<BevyBlockTypeStorageAsset>,
    #[asset(path = "global.materials.json")]
    pub materials_dict_asset: Handle<MaterialsDictAsset>,
}
