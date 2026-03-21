use bevy::{
    ecs::system::ResMut,
    log::{self, warn},
};
use shared::entities::{
    Chunk, ChunkPos, ChunkPosGenerator2D, ChunkPosGenerator3D, ChunkRepository,
};
use std::fmt;

use crate::chunk_manager::{ChunkStorage, ControllerPos};

use super::{chunk_state, ChunkState, ChunkStatus, ChunkTransition};

use super::physical_world::PhysicalWorld;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Horizontal radius (in chunks) within which chunks are loaded.
    pub load_distance: usize,
    /// Horizontal radius (in chunks) within which chunks are rendered.
    pub render_distance: usize,
    /// If true, the engine dynamically loads chunks above and below the player based on vertical position.
    pub dynamic_vertical_loading: bool,
}

impl Config {
    pub fn new(load_distance: usize, render_distance: usize) -> Self {
        assert!(render_distance <= load_distance);

        Config {
            load_distance,
            render_distance,
            dynamic_vertical_loading: false,
        }
    }
}

/// Manage chunks dependent on controller position.
pub struct ChunkManager {
    world: PhysicalWorld,
    config: Config,
}

impl ChunkManager {
    pub fn new(config: Config) -> Self {
        ChunkManager {
            world: PhysicalWorld::default(),
            config,
        }
    }

    /// Updates the controller position and triggers chunk state updates if position has changed.
    pub fn update_controller_pos(
        &mut self,
        mut controller_pos: ResMut<ControllerPos>,
        new_controller_pos: ChunkPos,
        chunks: &mut ChunkStorage,
    ) {
        if new_controller_pos != controller_pos.0 {
            controller_pos.0 = new_controller_pos;
            self.update_chunk_states_in_world(chunks, &controller_pos);
        }
    }

    pub fn get_world(&self) -> &PhysicalWorld {
        &self.world
    }

    fn update_chunk_states_in_world(
        &mut self,
        chunks: &mut ChunkStorage,
        controller_pos: &ControllerPos,
    ) {
        // Update all existing chunks in the world.
        let chunks_in_world: Vec<ChunkPos> = self.world.chunk_states.keys().copied().collect();

        for pos in chunks_in_world {
            self.update_chunk_state(pos, chunks, controller_pos);
        }

        // Remove all chunks that are still empty.
        let empty_chunks_in_world: Vec<ChunkPos> =
            self.world.get_chunks_with_state(ChunkState::Empty);

        for pos in empty_chunks_in_world {
            self.world.remove_chunk(pos);
            chunks.remove_chunk(pos);
        }
    }

    fn is_in_world_scope(&self, pos: ChunkPos) -> bool {
        if !self.config.dynamic_vertical_loading && pos.z != 0 {
            return false;
        }

        true
    }

    /// Always use this instead of set_chunk_state directly – handles transitions.
    fn change_chunk_state(
        &mut self,
        pos: ChunkPos,
        new_state: ChunkState,
        chunks: &mut ChunkStorage,
    ) {
        let prev_state = self.world.get_chunk_state(pos);

        if prev_state != new_state {
            let transition = chunk_state::get_chunk_transition(prev_state, new_state);

            assert!(
                transition.is_some(),
                "Unsupported transition in chunk {pos:?}: {prev_state:?} -> {new_state:?}"
            );

            self.apply_transition(pos, transition.unwrap(), chunks);
            self.world.set_chunk_state(pos, new_state);
        }
    }

    fn update_chunk_state(
        &mut self,
        pos: ChunkPos,
        chunks: &mut ChunkStorage,
        controller_pos: &ControllerPos,
    ) {
        const MAX_ITERATIONS: u32 = 16;
        let mut iterations = 1;
        let mut prev_state = self.world.get_chunk_state(pos);

        loop {
            let chunk_status = self.create_chunk_status(pos, chunks, controller_pos);
            let next_state = chunk_state::get_next_chunk_state(prev_state, chunk_status);

            if next_state == prev_state {
                break;
            }

            self.change_chunk_state(pos, next_state, chunks);
            log::trace!("Chunk {:?}: {:?} -> {:?}", pos, prev_state, next_state);

            if iterations >= MAX_ITERATIONS {
                log::warn!(
                    "Chunk {:?} failed to stabilize state after {} iterations",
                    pos,
                    MAX_ITERATIONS
                );
                break;
            }

            prev_state = next_state;
            iterations += 1;
        }
    }

    fn is_within_distance(
        &self,
        pos: ChunkPos,
        distance_in_chunks: usize,
        controller_pos: &ControllerPos,
    ) -> bool {
        match self.config.dynamic_vertical_loading {
            true => pos.is_within_distance(controller_pos.0, distance_in_chunks),
            false => pos.is_within_distance_2d(controller_pos.0, distance_in_chunks),
        }
    }

    fn apply_transition(
        &mut self,
        pos: ChunkPos,
        transition: ChunkTransition,
        chunks: &mut ChunkStorage,
    ) {
        use ChunkTransition::*;

        match transition {
            EmptyToLoading => {}
            LoadingToEmpty => {}
            LoadingToLoaded => {
                log::debug!("Loaded chunk {:?}", pos);
            }
            LoadedToEmpty => {
                chunks.remove_chunk(pos);
                self.world.remove_chunk(pos);
            }
        }
    }

    fn create_chunk_status(
        &self,
        pos: ChunkPos,
        chunks: &ChunkStorage,
        controller_pos: &ControllerPos,
    ) -> ChunkStatus {
        let is_within_load =
            self.is_within_distance(pos, self.config.load_distance, controller_pos);
        let loaded = chunks.get_chunk(pos).is_some();

        ChunkStatus {
            is_within_load,
            loaded,
        }
    }
}

impl fmt::Debug for ChunkManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChunkManager")
            .field("world", &self.world)
            // .field("chunk_builder", &self.chunk_builder)
            .finish()
    }
}
