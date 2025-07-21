use bevy::prelude::*;

use crate::chunk_mesh_builder::ChunkMesh;
use shared::entities::{types::*, Chunk};

/// Chunks loaded by `ChunkLoader`, not added to world.
#[derive(Event, Debug)]
pub struct ChunkLoaded {
    pub chunk_pos: ChunkPos,
    pub chunk: Chunk,
}

/// Chunks requested to load by `ChunkLoader`.
#[derive(Event, Debug)]
pub struct ChunkLoadRequest {
    pub chunk_pos: ChunkPos,
}

/// Chunk meshes built by `ChunkBuilder`, not added to world.
#[derive(Event, Debug)]
pub struct ChunkBuilt {
    pub chunk_pos: ChunkPos,
    pub chunk_mesh: ChunkMesh,
}

/// Chunks requested to build by `ChunkBuilder`.
#[derive(Event, Debug)]
pub struct ChunkBuildRequest {
    pub chunk_pos: ChunkPos,
    pub chunk: Chunk,
}

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
