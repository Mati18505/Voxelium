use bevy::prelude::*;

use crate::{
    chunk_mesh_builder::ChunkMesh,
};
use shared::entities::*;

use super::chunk_state::*;

/// `chunk_streamer` requests to change world state.
#[derive(Event, Debug)]
pub enum ChunkStreamerRequest {
    /// Request to update the chunk state based on its status.
    Update(ChunkPos),
    /// Request to load the chunk if it is empty.
    Load(ChunkPos),
    /// Request to remove chunk.
    Remove(ChunkPos),
}

/// Chunks requested to load by `ChunkLoader`.
#[derive(Event, Debug)]
pub enum ChunkLoaderRequest {
    Load(ChunkPos),
    CancelLoading(ChunkPos),
}

/// Chunks loaded by `ChunkLoader`, not added to world.
#[derive(Event, Debug)]
pub struct ChunkLoaded {
    pub chunk_pos: ChunkPos,
    pub chunk: Chunk,
}