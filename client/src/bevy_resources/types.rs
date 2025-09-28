use crate::{bevy_resources::{Dictionary, MaterialHandle, Storage}, chunk_mesh_builder::{RenderShape, TextureIndex}};

pub type MaterialName = String;
pub type TextureName = String;

pub type TextureIndexDictionary = Dictionary<TextureName, TextureIndex>;

pub type MaterialStorage = Storage<MaterialHandle>;
pub type RenderShapeStorage = Storage<RenderShape>;