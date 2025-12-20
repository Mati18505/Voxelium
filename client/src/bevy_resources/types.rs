use bevy::{asset::Handle, image::Image};
use shared::entities::name_to_block_id;

use crate::{
    bevy_resources::{
        Dictionary, MaterialAsset, MaterialHandle, RenderDesc, Storage, TextureAsset,
    },
    chunk_mesh_builder::{RenderShape, TextureIndex},
};

pub type MaterialName = String;
pub type TextureName = String;
pub type BlockTypeName = String;
pub type TextureId = u32;

// Assets
pub type MaterialsDictionary = Dictionary<MaterialName, MaterialAsset>;
pub type RenderDescDictionary = Dictionary<BlockTypeName, RenderDesc>;
pub type TextureDictionary = Dictionary<TextureName, TextureAsset>;

/// Each [`TextureAsset::TextureArray `] have one [`TextureIndexDictionary`].
pub type TextureIndexDictionary = Dictionary<TextureName, TextureIndex>;

// Resources
pub type MaterialStorage = Storage<MaterialHandle>;
pub type RenderShapeStorage = Storage<RenderShape>;
pub type TextureIdStorage = Storage<Handle<Image>>;

impl RenderDescDictionary {
    /// Creates [`RenderShapeStorage`] from [`RenderDescDictionary`], by compiling each [`RenderShape`] from [`RenderDesc`].
    /// Items in `RenderShapeStorage` are in order defined by `block_registry`.
    pub fn compile(&self, texture_dictionary: &TextureIndexDictionary) -> RenderShapeStorage {
        let mut compiled: Vec<(u8, RenderShape)> = self
            .iter()
            .map(|(block_type_name, render_desc)| {
                let block_id = name_to_block_id(&block_type_name);
                let compiled = render_desc.compile(texture_dictionary);

                (block_id, compiled)
            })
            .collect();

        compiled.sort_by_key(|(id, _)| *id);

        let render_shapes = compiled.into_iter().map(|(_, shape)| shape).collect();

        RenderShapeStorage::new(render_shapes)
    }
}

// impl TextureDictionary {
//     pub fn compile(&self) -> TextureIdStorage {
//         let mut compiled: Vec<(u8, RenderShape)> = self
//             .iter()
//             .map(|(block_type_name, render_desc)| {
//                 let block_id = name_to_block_id(&block_type_name);
//                 let compiled = render_desc.compile(texture_dictionary);
//
//                 (block_id, compiled)
//             })
//             .collect();
//
//         compiled.sort_by_key(|(id, _)| *id);
//
//         let render_shapes = compiled.into_iter().map(|(_, shape)| shape).collect();
//
//         RenderShapeStorage::new(render_shapes)
//     }
// }
//
// impl From<TextureDictionary> for Vec<(String, TextureId)> {
//     fn from(resource: BevyBlockTypeStorageResource) -> Self {
//         resource
//             .blocks
//             .into_iter()
//             .enumerate()
//             .map(|(i, e)| (e.name, i as BlockID))
//             .collect()
//     }
// }
