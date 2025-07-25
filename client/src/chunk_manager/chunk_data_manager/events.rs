use bevy::prelude::*;

use crate::chunk_mesh_builder::ChunkMesh;
use shared::entities::*;

use super::chunk_state::*;

/// `chunk_streamer` requests to update chunks in load distance.
#[derive(Event, Debug)]
pub struct ChunkStreamerRequest {
    pub chunk_pos: ChunkPos,
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
