use shared::entities::name_to_block_id;

use crate::{bevy_resources::{Dictionary, MaterialAsset, MaterialHandle, RenderDesc, Storage}, chunk_mesh_builder::{RenderShape, TextureIndex}};

pub type MaterialName = String;
pub type TextureName = String;
pub type BlockTypeName = String;

pub type MaterialsDictionary = Dictionary<MaterialName, MaterialAsset>;
pub type RenderDescDictionary = Dictionary<BlockTypeName, RenderDesc>;
pub type TextureIndexDictionary = Dictionary<TextureName, TextureIndex>;

pub type MaterialStorage = Storage<MaterialHandle>;
pub type RenderShapeStorage = Storage<RenderShape>;

/// Creates [`RenderShapeStorage`] from [`RenderDescDictionary`], by compiling each [`RenderShape`] from [`RenderDesc`].
/// Items in `RenderShapeStorage` are in order defined by `block_registry`.
pub fn compile_render_desc_dict(render_desc_dict: &RenderDescDictionary, texture_dictionary: &TextureIndexDictionary) -> RenderShapeStorage {
    let mut compiled: Vec<(u8, RenderShape)> = render_desc_dict
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