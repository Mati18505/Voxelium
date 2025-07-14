use std::{collections::{HashMap, HashSet}, fmt};

use shared::entities::{world::World, Chunk, ChunkPos, ChunkRepository};

use crate::chunk_mesh_builder::ChunkMesh;

use super::ChunkState;

pub type Version = u64;

#[derive(Default, Clone, PartialEq)]
pub struct PhysicalWorld {
    pub world: World,
    pub chunk_meshes: HashMap<ChunkPos, ChunkMesh>,
    pub chunk_states: HashMap<ChunkPos, ChunkState>,
    chunk_mesh_versions: HashMap<ChunkPos, Version>,
    chunks_need_rebuild: HashSet<ChunkPos>,
}

impl PhysicalWorld {
    pub fn add_chunk_mesh(&mut self, pos: ChunkPos, chunk_mesh: ChunkMesh) {
        self.chunk_meshes.insert(pos, chunk_mesh);
    }
    pub fn get_chunk_mesh(&self, pos: ChunkPos) -> Option<&ChunkMesh> {
        self.chunk_meshes.get(&pos)
    }

    pub fn set_chunk_state(&mut self, pos: ChunkPos, state: ChunkState) {
        self.chunk_states.insert(pos, state);
    }
    pub fn get_chunk_state(&self, pos: ChunkPos) -> ChunkState {
        self.chunk_states.get(&pos).copied().unwrap_or(ChunkState::Empty)
    }

    pub fn increment_chunk_mesh_version(&mut self, pos: ChunkPos) -> Version {
        let incremented_version = *self.chunk_mesh_versions.entry(pos).and_modify(|e| *e = e.wrapping_add(1)).or_insert(1);
        Version::from(incremented_version)
    }

    pub fn get_chunk_mesh_version(&self, pos: ChunkPos) -> Version {
        self.chunk_mesh_versions.get(&pos).copied().unwrap_or(0)
    }

    pub fn set_chunk_need_rebuild(&mut self, pos: ChunkPos) {
        self.chunks_need_rebuild.insert(pos);
    }

    pub fn get_chunk_need_rebuild(&self, pos: ChunkPos) -> bool {
        self.chunks_need_rebuild.contains(&pos)
    }

    pub fn remove_chunk_need_rebuild(&mut self, pos: ChunkPos) {
        self.chunks_need_rebuild.remove(&pos);
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
        self.chunk_mesh_versions.remove(&pos);
        self.world.remove_chunk(pos);
    }

    fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.world.get_chunk(pos)
    }
}

impl fmt::Debug for PhysicalWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let empty = self.get_chunks_with_state::<Vec<ChunkPos>>(ChunkState::Empty).len();
        let loading = self.get_chunks_with_state::<Vec<ChunkPos>>(ChunkState::Loading).len();
        let loaded = self.get_chunks_with_state::<Vec<ChunkPos>>(ChunkState::Loaded).len();
        let to_draw = self.get_chunks_with_state::<Vec<ChunkPos>>(ChunkState::ToDraw).len();
        let drawn = self.get_chunks_with_state::<Vec<ChunkPos>>(ChunkState::Drawn).len();
        let chunks_need_rebuild = self.chunks_need_rebuild.len();

        f.debug_struct("PhysicalWorld")
            .field("chunks", &self.world.chunks.len())
            .field("chunk_meshes", &self.chunk_meshes.len())
            .field("chunk_states", &self.chunk_states.len())
            .field("chunk_mesh_versions", &self.chunk_mesh_versions.len())
            .field("empty", &empty)
            .field("loading", &loading)
            .field("loaded", &loaded)
            .field("to_draw", &to_draw)
            .field("drawn", &drawn)
            .field("chunks_need_rebuild", &chunks_need_rebuild)
            .finish()
    }
}