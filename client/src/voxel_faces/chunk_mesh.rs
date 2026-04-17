use bevy::mesh::{Meshable};
use shared::entities::{BlockInChunkPos, BlockSide};

use crate::chunk_mesh_builder::meshers::ChunkMeshBuilder;

pub type StorageIndex = u32;

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
