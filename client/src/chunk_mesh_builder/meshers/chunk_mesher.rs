use std::fmt::Debug;
use thiserror::Error;

use crate::chunk_mesh_builder::{ChunkMesh, TextureName};
use shared::entities::BlockID;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum MesherWarning {
    #[error("Mesher encountered unknown block type id: {0}, on position: {1}")]
    UnknownBlockType(BlockID, BlockPos),

    #[error("Mesher encountered unknown texture name: {0}, on position: {1}")]
    UnknownTextureName(TextureName, BlockPos),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct MesherOutput {
    /// Generated chunk mesh.
    mesh: ChunkMesh,
    /// Non-fatal issues encountered during mesh creation (may contain duplicates).
    warnings: Vec<MesherWarning>,
}

pub trait ChunkMesher: Debug {
    /// Creates chunk mesh based on its data.
    /// Mesh is always created to the end, warnings don't interrupt mesh creation.
    fn create_mesh(&self, chunk: &Chunk) -> MesherOutput;
}
