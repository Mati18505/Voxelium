use std::{collections::HashMap, fmt::Debug};
use bevy::{math::primitives::Plane3d, mesh::Meshable};
use thiserror::Error;

use shared::entities::{BlockID, BlockInChunkPos, BlockSide, Chunk};

use crate::chunk_mesh_builder::ChunkMeshBuilder;

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum MesherWarning {
    #[error("Mesher encountered unknown render shape id: {0}")]
    UnknownRenderShape(BlockID),
}

#[derive(Debug, Clone)]
pub struct Quad {
    pub facing_side: BlockSide,
    pub block_pos: BlockInChunkPos,
}

#[derive(Debug, Clone)]
pub struct ChunkMeshData {
    pub quads: Vec<Quad>,
}

impl Default for ChunkMeshData {
    fn default() -> Self {
        Self {
            quads: Vec::new()
        }
    }
}

impl Meshable for ChunkMeshData {
    type Output = ChunkMeshBuilder;

    fn mesh(&self) -> Self::Output {
        ChunkMeshBuilder {
            chunk_mesh_data: self.clone(),
        }
    }
}

pub type MesherWarnings = HashMap<MesherWarning, u32>;

#[derive(Debug, Default, Clone)]
pub struct MesherOutput {
    /// Generated chunk mesh.
    pub mesh: ChunkMeshData,
    /// Non-fatal issues encountered during mesh creation and repetition count.
    pub warnings: MesherWarnings,
}

pub trait ChunkMesher: Send + Sync + Debug {
    /// Creates chunk mesh based on its data.
    /// Mesh is always created to the end, warnings don't interrupt mesh creation.
    fn create_mesh(&self, chunk: &Chunk) -> MesherOutput;
}
