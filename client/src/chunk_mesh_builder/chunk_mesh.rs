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
    pub layers: HashMap<MaterialId, Mesh>,
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
