use std::fmt::Debug;
use thiserror::Error;

use crate::chunk_mesh_builder::ChunkMesh;
use shared::entities::{BlockID, BlockInChunkPos, Chunk};

#[derive(Debug, Error, Clone, PartialEq)]
pub enum MesherWarning {
    #[error("Mesher encountered unknown render shape id: {0}, on position: {1:?}")]
    UnknownRenderShape(BlockID, BlockInChunkPos),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MesherOutput {
    /// Generated chunk mesh.
    pub mesh: ChunkMesh,
    /// Non-fatal issues encountered during mesh creation (may contain duplicates).
    pub warnings: Vec<MesherWarning>,
}

pub trait ChunkMesher: Send + Sync + Debug {
    /// Creates chunk mesh based on its data.
    /// Mesh is always created to the end, warnings don't interrupt mesh creation.
    fn create_mesh(&self, chunk: &Chunk) -> MesherOutput;
}
