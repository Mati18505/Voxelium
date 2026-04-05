pub mod asset_plugin;
pub use asset_plugin::*;
pub use blocks_asset::BlockTypeStorageAsset;
pub use material_asset::*;
pub use materials_dict_asset::MaterialsDictAsset;
pub use render_desc_dict_asset::RenderDescDictAsset;
pub use texture_asset::*;
pub use texture_dict_asset::TextureDictAsset;

mod blocks_asset;
mod chunk_loader_asset;
mod loaders;
mod material_asset;
mod materials_dict_asset;
mod render_desc_dict_asset;
mod texture_asset;
mod texture_dict_asset;
mod textured_block_type_builder;
