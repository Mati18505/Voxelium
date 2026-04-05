use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use super::blocks_asset::{BlockTypeStorageAsset, BlocksAssetPlugin};
use super::chunk_builder_asset::{ChunkBuilderAssetPlugin, ChunkBuilderConfigAsset};
use super::chunk_loader_asset::{ChunkLoaderAssetPlugin, ChunkLoaderConfigAsset};
use super::controller_asset::{ControllerAssetPlugin, ControllerConfigAsset};
use super::materials::{MaterialsDictAsset, MaterialsDictAssetPlugin};
use super::render_desc::{RenderDescAssetPlugin, RenderDescDictAsset};
use super::textures::{TextureAssetPlugin, TextureDictAsset};

use crate::bevy_types::AppStates;

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChunkBuilderAssetPlugin,
            ChunkLoaderAssetPlugin,
            BlocksAssetPlugin,
            RenderDescAssetPlugin,
            MaterialsDictAssetPlugin,
            TextureAssetPlugin,
            ControllerAssetPlugin,
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
    #[asset(path = "config.chunk_builder.yaml")]
    pub chunk_builder_settings: Handle<ChunkBuilderConfigAsset>,
    #[asset(path = "config.controller.yaml")]
    pub controller_settings: Handle<ControllerConfigAsset>,
}

#[derive(AssetCollection, Resource)]
pub struct VoxelAssets {
    #[asset(path = "global.render_desc.json")]
    pub render_desc_storage_res: Handle<RenderDescDictAsset>,
    #[asset(path = "global.textures.yaml")]
    pub texture_dict_asset: Handle<TextureDictAsset>,
    #[asset(path = "global.blocks.json")]
    pub server_blocks: Handle<BlockTypeStorageAsset>,
    #[asset(path = "global.materials.json")]
    pub materials_dict_asset: Handle<MaterialsDictAsset>,
}
