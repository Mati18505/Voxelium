use std::collections::HashMap;

use bevy::{
    mesh::Mesh,
    transform::components::Transform,
};

use crate::chunk_mesh_builder::{ChunkMesh, MaterialId};

#[derive(Debug, Default, Clone)]
pub struct BevyChunkMesh {
    pub layers: HashMap<MaterialId, Mesh>,
    pub transform: Transform,
}

impl BevyChunkMesh {
    pub fn apply_transform(&mut self, transform: Transform) {
        self.transform = self.transform * transform;
    }
}

impl From<ChunkMesh> for BevyChunkMesh {
    fn from(chunk_mesh: ChunkMesh) -> Self {
        BevyChunkMesh {
            layers: chunk_mesh.layers,
            transform: Transform::IDENTITY,
        }
    }
}
