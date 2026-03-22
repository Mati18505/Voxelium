use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;

use shared::entities::BlockID;

use crate::chunk_mesh_builder::{ChunkMeshData, ChunkWithBorder, MaterialId};

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum MesherWarning {
    #[error("Mesher encountered unknown render shape id: {0}")]
    UnknownRenderShape(BlockID),
}

pub type MesherWarnings = HashMap<MesherWarning, u32>;

#[derive(Debug, Default, Clone)]
pub struct MesherOutput {
    /// Generated chunk mesh data.
    pub layers: HashMap<MaterialId, ChunkMeshData>,
    /// Non-fatal issues encountered during mesh creation and repetition count.
    pub warnings: MesherWarnings,
}

pub trait ChunkMesher: Send + Sync + Debug {
    /// Creates chunk mesh data based on its data.
    /// Mesh is always created to the end, warnings don't interrupt mesh creation.
    fn create_mesh(&self, chunk: &ChunkWithBorder) -> MesherOutput;
}
