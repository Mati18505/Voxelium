use std::collections::HashMap;

use shared::entities::{world::World, ChunkPos};

use crate::chunk_builder::ChunkMesh;

use super::ChunkState;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PhysicalWorld {
    pub world: World,
    pub chunk_meshes: HashMap<ChunkPos, ChunkMesh>,
    pub chunk_states: HashMap<ChunkPos, ChunkState>,
}

impl PhysicalWorld {
    pub fn add_chunk_mesh(&mut self, pos: ChunkPos, chunk_mesh: ChunkMesh) {
        self.chunk_meshes.insert(pos, chunk_mesh);
    }
    pub fn get_chunk_mesh(&self, pos: ChunkPos) -> Option<&ChunkMesh> {
        self.chunk_meshes.get(&pos)
    }

    pub fn change_chunk_state(&mut self, pos: ChunkPos, state: ChunkState) {
        assert!(self.world.get_chunk(pos) != None, "Cannot change state of nonexistent chunk.");

        self.chunk_states.insert(pos, state);
    }
}