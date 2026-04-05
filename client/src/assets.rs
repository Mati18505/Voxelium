pub mod materials;
pub mod render_desc;
pub mod textures;
pub use asset_plugin::{AssetsPlugin, Config, VoxelAssets};
pub use blocks_asset::BlockTypeStorageAsset;

mod asset_plugin;
mod blocks_asset;
mod chunk_loader_asset;
mod textured_block_type_builder;
