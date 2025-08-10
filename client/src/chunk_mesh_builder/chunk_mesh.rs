use std::collections::HashMap;

pub type TextureIndex = u32;
pub type PaletteIndex = u32;
pub type StorageIndex = u32;
pub type MaterialId = u8;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LayerMesh {
    pub vertices: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub triangles: Vec<usize>,
    pub normals: Vec<[isize; 3]>,
    pub vertex_index: usize,
    pub indexes: Vec<StorageIndex>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ChunkMesh {
    pub layers: HashMap<MaterialId, LayerMesh>,
}
