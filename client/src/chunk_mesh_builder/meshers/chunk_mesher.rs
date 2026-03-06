use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;

use crate::chunk_mesh_builder::ChunkMesh;
use shared::entities::{BlockID, BlockInChunkPos, Chunk};

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum MesherWarning {
    #[error("Mesher encountered unknown render shape id: {0}")]
    UnknownRenderShape(BlockID),
}

#[derive(Debug, Default, Clone)]
pub struct MesherOutput {
    /// Generated chunk mesh.
    pub mesh: ChunkMesh,
    /// Non-fatal issues encountered during mesh creation and repetition count.
    pub warnings: HashMap<MesherWarning, u32>,
}

pub trait ChunkMesher: Send + Sync + Debug {
    /// Creates chunk mesh based on its data.
    /// Mesh is always created to the end, warnings don't interrupt mesh creation.
    fn create_mesh(&self, chunk: &Chunk) -> MesherOutput;
}
