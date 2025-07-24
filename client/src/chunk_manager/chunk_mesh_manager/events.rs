use bevy::prelude::*;

use crate::{
    chunk_manager::{
        chunk_state::{self, ChunkTransition},
        ChunkState, ChunkStatus,
    },
    chunk_mesh_builder::ChunkMesh,
};
use shared::entities::{types::*, Chunk};

/// Chunks passed to `chunk_entities_manager` to create an entity.
#[derive(Event, Debug)]
pub enum ChunkEntityEvent {
    Create(ChunkPos, ChunkMesh),
    Remove(ChunkPos),
}

/// Chunks requested to build by `ChunkBuilder`.
#[derive(Event, Debug)]
pub enum ChunkBuilderRequest {
    Build(ChunkPos, Chunk),
    CancelBuilding(ChunkPos),
}

/// Chunk meshes built by `ChunkBuilder`, not added to world.
#[derive(Event, Debug)]
pub struct ChunkBuilt {
    pub chunk_pos: ChunkPos,
    pub chunk_mesh: ChunkMesh,
}