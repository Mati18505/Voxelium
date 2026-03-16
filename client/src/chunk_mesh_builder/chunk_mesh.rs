use std::collections::HashMap;

use bevy::mesh::Mesh;

pub type TextureIndex = u32;
pub type ColorIndex = u32;
pub type StorageIndex = u32;
pub type MaterialId = u8;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ChunkMesh {
    pub layers: HashMap<MaterialId, Mesh>,
}
