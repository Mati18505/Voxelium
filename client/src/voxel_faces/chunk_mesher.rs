use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;

use shared::entities::BlockID;

use crate::{
    chunk_mesh_builder::MaterialId,
    voxel_render_core::FaceData,
};

use super::ChunkWithNeighbors;

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum MesherWarning {
    #[error("Mesher encountered unknown render shape id: {0}")]
    UnknownRenderShape(BlockID),
}

pub type MesherWarnings = HashMap<MesherWarning, u32>;

#[derive(Debug, Default, Clone)]
pub struct MesherOutput {
    /// Generated chunk faces.
    pub layers: HashMap<MaterialId, Vec<FaceData>>,
    /// Non-fatal issues encountered during mesh creation and repetition count.
    pub warnings: MesherWarnings,
}

pub trait ChunkMesher: Send + Sync + Debug {
    /// Generates chunk faces based on its data.
    /// Faces are always generated to the end, warnings don't interrupt face generation.
    fn create_mesh(&self, chunk: &ChunkWithNeighbors) -> MesherOutput;
}
