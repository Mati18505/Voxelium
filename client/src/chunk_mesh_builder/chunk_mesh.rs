use std::collections::HashMap;

use bevy::mesh::{Mesh, Meshable};
use shared::entities::{BlockInChunkPos, BlockSide};

use crate::chunk_mesh_builder::meshers::ChunkMeshBuilder;

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

#[derive(Debug, Clone)]
pub struct FaceData {
    pub facing_side: BlockSide,
    pub block_pos: BlockInChunkPos,
    pub uv_2: StorageIndex,
}

#[derive(Debug, Default, Clone)]
pub struct ChunkMeshData {
    pub faces: Vec<FaceData>,
}

impl Meshable for ChunkMeshData {
    type Output = ChunkMeshBuilder;

    fn mesh(&self) -> Self::Output {
        ChunkMeshBuilder {
            chunk_mesh_data: self.clone(),
        }
    }
}
