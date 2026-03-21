use std::{collections::HashMap, fmt};

use shared::entities::ChunkPos;

use super::ChunkState;

#[derive(Default, Clone, PartialEq)]
pub struct PhysicalWorld {
    pub chunk_states: HashMap<ChunkPos, ChunkState>,
}

impl PhysicalWorld {
    pub fn set_chunk_state(&mut self, pos: ChunkPos, state: ChunkState) {
        self.chunk_states.insert(pos, state);
    }
    pub fn get_chunk_state(&self, pos: ChunkPos) -> ChunkState {
        self.chunk_states
            .get(&pos)
            .copied()
            .unwrap_or(ChunkState::Empty)
    }

    pub fn get_chunks_with_state<T: FromIterator<ChunkPos>>(&self, state: super::ChunkState) -> T {
        self.chunk_states
            .iter()
            .filter(|(_, chunk_state)| **chunk_state == state)
            .map(|(chunk_pos, _)| *chunk_pos)
            .collect()
    }

    pub fn remove_chunk(&mut self, pos: ChunkPos) {
        self.chunk_states.remove(&pos);
    }
}

impl fmt::Debug for PhysicalWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let empty = self
            .get_chunks_with_state::<Vec<ChunkPos>>(ChunkState::Empty)
            .len();
        let loading = self
            .get_chunks_with_state::<Vec<ChunkPos>>(ChunkState::Loading)
            .len();
        let loaded = self
            .get_chunks_with_state::<Vec<ChunkPos>>(ChunkState::Loaded)
            .len();

        f.debug_struct("PhysicalWorld")
            .field("chunk_states", &self.chunk_states.len())
            .field("empty", &empty)
            .field("loading", &loading)
            .field("loaded", &loaded)
            .finish()
    }
}
