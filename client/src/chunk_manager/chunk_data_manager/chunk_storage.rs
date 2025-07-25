use bevy::prelude::*;
use std::{collections::HashMap, fmt::Debug};

use shared::entities::{Chunk, ChunkPos, ChunkRepository, World};

use super::chunk_state::{ChunkState, ChunkTransition};

/// Storage of chunk data and its state.
#[derive(Resource, Default)]
pub struct ChunkStorage {
    world: World,
    chunk_states: HashMap<ChunkPos, ChunkState>,
}

impl ChunkStorage {
    /// Returns true if chunk is in storage and its state is Loaded.
    /// Else returns false.
    pub fn is_loaded(&self, pos: ChunkPos) -> bool {
        let chunk_in_storage = self.get_chunk(pos).is_some();
        let state_is_loaded = self.get_chunk_state(pos) == ChunkState::Loaded;

        assert_eq!(
            chunk_in_storage, state_is_loaded,
            "Chunk-state mismatch at {:?}",
            pos
        );

        match (chunk_in_storage, state_is_loaded) {
            (true, true) => true,
            (false, false) => false,
            // State mismatch.
            _ => unreachable!(),
        }
    }

    /// Inserts chunk into storage only if transition from current state to ChunkState::Loaded is allowed.
    pub fn load(&mut self, pos: ChunkPos, chunk: Chunk) {
        let transition_allowed = self
            .get_chunk_state(pos)
            .get_chunk_transition(ChunkState::Loaded)
            .is_some();
        assert!(transition_allowed);

        self.chunk_states.insert(pos, ChunkState::Loaded);
        self.world.chunks.insert(pos, chunk);
    }

    /// Changes the chunk state based on state transition.
    /// If current chunk state is different than that of the transition, function panics.
    /// If `transition.to` is equal to ChunkState::Loaded - this function panics - instead use method Load.
    pub fn change_state(&mut self, pos: ChunkPos, transition: ChunkTransition) {
        assert_eq!(transition.from, self.get_chunk_state(pos));
        assert_ne!(transition.to, ChunkState::Loaded);

        if transition.to == ChunkState::Empty {
            self.remove_chunk(pos);
        } else {
            self.chunk_states.insert(pos, transition.to);
        }
    }

    /// Returns chunk state.
    /// If it does not exist returns ChunkState::Empty.
    pub fn get_chunk_state(&self, pos: ChunkPos) -> ChunkState {
        self.chunk_states
            .get(&pos)
            .copied()
            .unwrap_or(ChunkState::Empty)
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.world.chunks.get(&pos)
    }
    pub fn contains_chunk(&self, pos: ChunkPos) -> bool {
        self.world.chunks.contains_key(&pos)
    }

    /// Returns iterator of all chunk positions in storage that chunk have state equal to provided.
    pub fn get_chunks_with_state(&self, state: ChunkState) -> impl Iterator<Item = ChunkPos> + '_ {
        self.chunk_states
            .iter()
            .filter(move |(_, chunk_state)| **chunk_state == state)
            .map(|(chunk_pos, _)| *chunk_pos)
    }

    /// Returns all chunk positions in storage.
    pub fn get_all_chunks(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.chunk_states.keys().copied()
    }

    /// Removes chunk from storage only if transition from current state to ChunkState::Empty is allowed.
    fn remove_chunk(&mut self, pos: ChunkPos) {
        let transition_allowed = self
            .get_chunk_state(pos)
            .get_chunk_transition(ChunkState::Empty)
            .is_some();
        assert!(transition_allowed);

        self.world.chunks.remove(&pos);
        self.chunk_states.remove(&pos);
    }
}

impl Debug for ChunkStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkStorage")
            .field("chunks", &self.world.chunks.len())
            .field("chunk_states", &self.chunk_states.len())
            .finish()
    }
}
