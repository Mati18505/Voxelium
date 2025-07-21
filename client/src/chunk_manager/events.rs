use bevy::prelude::*;

use crate::{
    chunk_manager::{
        chunk_state::{self, ChunkTransition},
        ChunkState, ChunkStatus,
    },
    chunk_mesh_builder::ChunkMesh,
};
use shared::entities::{types::*, Chunk};

/// Chunks loaded by `ChunkLoader`, not added to world.
#[derive(Event, Debug)]
pub struct ChunkLoaded {
    pub chunk_pos: ChunkPos,
    pub chunk: Chunk,
}

/// Chunks requested to load by `ChunkLoader`.
#[derive(Event, Debug)]
pub enum ChunkLoaderRequest {
    Load(ChunkPos),
    CancelLoading(ChunkPos),
}

/// Chunk meshes built by `ChunkBuilder`, not added to world.
#[derive(Event, Debug)]
pub struct ChunkBuilt {
    pub chunk_pos: ChunkPos,
    pub chunk_mesh: ChunkMesh,
}

/// Chunks requested to build by `ChunkBuilder`.
#[derive(Event, Debug)]
pub enum ChunkBuilderRequest {
    Build(ChunkPos, Chunk),
    CancelBuilding(ChunkPos),
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

/// `world_event_handler` request to update the chunk state based on its status.
#[derive(Event, Debug)]
pub struct StateUpdateRequest {
    pub chunk_pos: ChunkPos,
    pub curr_state: ChunkState,
    pub chunk_status: ChunkStatus,
}

/// Chunk is changing it's state.
#[derive(Event, Debug)]
pub struct ChunkStateTransition {
    pub chunk_pos: ChunkPos,
    pub transition: ChunkTransition,
    pub new_state: ChunkState,
}

/// Chunks passed to `chunk_entities_manager`to create an entity.
#[derive(Event, Debug)]
pub enum ChunkEntityEvent {
    Create(ChunkPos, ChunkMesh),
    Remove(ChunkPos),
}
