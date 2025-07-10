use std::collections::HashMap;

use shared::entities::{world::World, Chunk, ChunkPos, ChunkRepository};

use crate::chunk_builder::ChunkMesh;

use super::ChunkState;

pub type Version = u64;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PhysicalWorld {
    pub world: World,
    pub chunk_meshes: HashMap<ChunkPos, ChunkMesh>,
    pub chunk_states: HashMap<ChunkPos, ChunkState>,
    chunk_mesh_versions: HashMap<ChunkPos, Version>,
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
    pub fn get_chunk_state(&self, pos: ChunkPos) -> Option<&ChunkState> {
        self.chunk_states.get(&pos)
    }

    pub fn increment_chunk_mesh_version(&mut self, pos: ChunkPos) -> Version {
        let incremented_version = *self.chunk_mesh_versions.entry(pos).and_modify(|e| *e = e.wrapping_add(1)).or_insert(1);
        Version::from(incremented_version)
    }

    pub fn get_chunk_mesh_version(&self, pos: ChunkPos) -> Version {
        self.chunk_mesh_versions.get(&pos).copied().unwrap_or(0)
    }

    pub fn get_chunks_with_state<T: FromIterator<ChunkPos>>(&self, state: super::ChunkState) -> T {
        self.chunk_states
            .iter()
            .filter(|(_, chunk_state)| **chunk_state == state)
            .map(|(chunk_pos, _)| *chunk_pos)
            .collect()
    }
}

impl ChunkRepository for PhysicalWorld {
    fn set_chunk(&mut self, pos: ChunkPos, new_chunk: Chunk) {
        self.world.set_chunk(pos, new_chunk);
    }

    fn remove_chunk(&mut self, pos: ChunkPos) {
        self.chunk_meshes.remove(&pos);
        self.chunk_states.remove(&pos);
        self.world.remove_chunk(pos);
    }

    fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.world.get_chunk(pos)
    }
}