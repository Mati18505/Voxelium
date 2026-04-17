use std::collections::HashMap;

use bevy::mesh::Mesh;

pub type TextureIndex = u32;
pub type ColorIndex = u32;
pub type StorageIndex = u32;
pub type MaterialId = u8;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ChunkMesh {
    layers: HashMap<MaterialId, Mesh>,
    /// Diagnosis.
    vertex_count: usize,
}

impl ChunkMesh {
    pub fn layers(&self) -> &HashMap<MaterialId, Mesh> {
        &self.layers
    }
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    pub fn add_layer(&mut self, material_id: MaterialId, mesh: Mesh) {
        self.vertex_count += mesh.count_vertices();
        self.layers.insert(material_id, mesh);
    }
}
