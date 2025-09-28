use crate::{bevy_resources::{Dictionary, MaterialHandle, RenderDesc, Storage}, chunk_mesh_builder::{RenderShape, TextureIndex}};

pub type MaterialName = String;
pub type TextureName = String;
pub type BlockTypeName = String;

pub type TextureIndexDictionary = Dictionary<TextureName, TextureIndex>;
pub type RenderDescDictionary = Dictionary<BlockTypeName, RenderDesc>;

pub type MaterialStorage = Storage<MaterialHandle>;
pub type RenderShapeStorage = Storage<RenderShape>;

pub fn compile_render_desc_dict(render_desc_dict: &RenderDescDictionary, texture_dictionary: &TextureIndexDictionary) -> RenderShapeStorage {
    let compiled: Vec<RenderShape> = render_desc_dict
        .iter()
        .map(|(block_type_name, render_desc)| render_desc.compile(texture_dictionary))
        .collect();

    RenderShapeStorage::new(compiled)
}