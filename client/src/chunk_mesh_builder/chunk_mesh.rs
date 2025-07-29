use std::collections::HashMap;

use super::TextureIndex;

pub type MaterialId = u8;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LayerMesh {
    pub vertices: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub texture_indexes: Vec<TextureIndex>,
    pub triangles: Vec<usize>,
    pub normals: Vec<[isize; 3]>,
    pub vertex_index: usize,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ChunkMesh {
    pub layers: HashMap<MaterialId, LayerMesh>,
}
