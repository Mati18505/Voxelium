use bevy::{
    ecs::{
        message::MessageWriter,
        system::{Res, ResMut},
    },
    log::{self, info_span},
};
use shared::{
    chunk_io::chunk_loader,
    entities::{Chunk, ChunkPos, ChunkPosGenerator2D, ChunkPosGenerator3D, ChunkRepository},
};
use std::fmt;

use crate::{
    chunk_manager::{
        chunk_builder::{self, BuildChunk},
        ChunkStorage, ControllerPos, RemoveChunk,
    },
    chunk_mesh_builder::{
        builders::{ChunkBuilder, Versioned},
        ChunkMesh,
    },
};

use super::{chunk_state, ChunkState, ChunkStatus, ChunkTransition};

use super::physical_world::PhysicalWorld;

#[derive(Debug, Clone, PartialEq)]
pub struct WorldChunkUpdate {
    pub chunk_pos: ChunkPos,
    pub chunk: Chunk,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkObjectEvent {
    Created(ChunkPos, ChunkMesh),
    Removed(ChunkPos),
}

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

pub trait VersionedChunkBuilder<T: Send + Sync + Default>: ChunkBuilder<T> + Versioned<T> {}

impl<T: Send + Sync + Default, U> VersionedChunkBuilder<T> for U where
    U: ChunkBuilder<T> + Versioned<T>
{
}

/// Manage chunks dependent on controller position.
pub struct ChunkManager {
    world: PhysicalWorld,
    chunk_loader: chunk_loader::ChunkLoader,
    chunk_object_tx: Option<crossbeam_channel::Sender<ChunkObjectEvent>>,
    event_tx: Option<crossbeam_channel::Sender<WorldChunkUpdate>>,
    config: Config,
    chunks_to_build: Vec<ChunkPos>,
}

impl ChunkManager {
    pub fn new(chunk_loader: chunk_loader::ChunkLoader, config: Config) -> Self {
        ChunkManager {
            world: PhysicalWorld::default(),
            chunk_loader,
            chunk_object_tx: None,
            event_tx: None,
            config,
            chunks_to_build: Vec::default(),
        }
    }

    /// Sets the callback used when a chunk is drawn or mesh is removed.
    pub fn set_chunk_object_tx(
        &mut self,
        callback: Option<crossbeam_channel::Sender<ChunkObjectEvent>>,
    ) {
        self.chunk_object_tx = callback;
    }

    /// Sets the callback used after chunk has been modified.
    pub fn set_event_tx(&mut self, callback: Option<crossbeam_channel::Sender<WorldChunkUpdate>>) {
        self.event_tx = callback;
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

    /// Processes chunks ready to be loaded.
    /// Should be called once per frame.
    pub fn check_loaded_chunks(
        &mut self,
        chunks: &mut ChunkStorage,
        controller_pos: &ControllerPos,
    ) {
        self.chunk_loader.update(controller_pos.0);
        let completed = self.chunk_loader.poll_loaded_chunks();

        for (pos, chunk) in completed {
            chunks.set_chunk(pos, chunk);
            self.update_chunk_state(pos, chunks, controller_pos);
        }
    }

    /// Checks and processes chunks ready to be drawn.
    /// Should be called once per frame.
    pub fn check_built_chunks(
        &mut self,
        chunks: &mut ChunkStorage,
        controller_pos: &ControllerPos,
    ) {
        let _ = info_span!("check_built_chunks", name = "check_built_chunks").entered();

        //self.chunk_builder.update(controller_pos.0);

        let chunks_to_draw: Vec<ChunkPos> = self.world.get_chunks_with_state(ChunkState::ToDraw);

        for chunk_pos in chunks_to_draw {
            self.update_chunk_state(chunk_pos, chunks, controller_pos);
        }
    }

    pub fn get_world(&self) -> &PhysicalWorld {
        &self.world
    }

    /// Gets chunk from the world or loads it if it is not loaded yet.
    /// Returns None only if the position is outside the world scope.
    #[allow(dead_code)]
    pub fn get_or_load_chunk(&mut self, pos: ChunkPos, chunks: &mut ChunkStorage) -> Option<Chunk> {
        if !self.is_in_world_scope(pos) {
            return None;
        }

        self.load_chunk_if_is_empty(pos, chunks);

        chunks.get_chunk(pos).cloned()
    }

    pub fn send_messages_to_builder(
        &mut self,
        chunks_to_build: &mut MessageWriter<BuildChunk>,
        chunks_to_remove: &mut MessageWriter<RemoveChunk>,
    ) {
        for chunk_pos in std::mem::take(&mut self.chunks_to_build) {
            chunks_to_build.write(BuildChunk(chunk_pos));
        }
    }

    fn update_chunk_states_in_world(
        &mut self,
        chunks: &mut ChunkStorage,
        controller_pos: &ControllerPos,
    ) {
        // Load missing chunks within the load distance.
        let generator: Box<dyn Iterator<Item = ChunkPos>> =
            match self.config.dynamic_vertical_loading {
                true => Box::new(ChunkPosGenerator3D::new(
                    controller_pos.0,
                    self.config.load_distance,
                )),
                false => Box::new(ChunkPosGenerator2D::new(
                    controller_pos.0,
                    self.config.load_distance,
                )),
            };

        for pos in generator {
            self.load_chunk_if_is_empty(pos, chunks);
        }

        // Update all existing chunks in the world.
        let chunks_in_world: Vec<ChunkPos> = self.world.chunk_states.keys().copied().collect();

        for pos in chunks_in_world {
            self.update_chunk_state(pos, chunks, controller_pos);
        }

        // Remove all chunks that are still empty.
        let empty_chunks_in_world: Vec<ChunkPos> =
            self.world.get_chunks_with_state(ChunkState::Empty);

        for pos in empty_chunks_in_world {
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
            EmptyToLoading => {
                self.chunk_loader.load_chunk(pos);
            }
            LoadingToEmpty => {
                self.chunk_loader.cancel_loading_chunk(pos);
            }
            LoadingToLoaded => {
                log::debug!("Loaded chunk {:?}", pos);
            }
            LoadedToEmpty => {
                chunks.remove_chunk(pos);
            }
            LoadedToToDraw => {
                self.pass_chunk_to_builder(pos);
            }
            ToDrawToLoaded => {
                // TODO: Remove mesh from chunk builder.
                // self.chunk_builder.remove_chunk(chunk_pos);
            }
            ToDrawToDrawn => {
                /*
                let mesh = self
                    .chunk_builder
                    .take_chunk_built_with_latest_version(pos)
                    .expect("ChunkState is drawn, but mesh is not built.")
                    .0;

                self.create_chunk_object(pos, &mesh);
                */
            }
            DrawnToToDraw => {
                self.pass_chunk_to_builder(pos);
            }
            DrawnToLoaded => {
                self.remove_chunk_object(pos);
            }
        }
    }

    fn pass_chunk_to_builder(&mut self, pos: ChunkPos) {
        self.chunks_to_build.push(pos);

        self.world.remove_chunk_need_rebuild(pos);
    }

    fn emit_event(&self, ev: WorldChunkUpdate) {
        if let Some(event_tx) = &self.event_tx {
            let _ = event_tx.send(ev);
        }
    }

    fn create_chunk_object(&self, pos: ChunkPos, mesh: &ChunkMesh) {
        if let Some(chunk_object_tx) = &self.chunk_object_tx {
            let _ = chunk_object_tx.send(ChunkObjectEvent::Created(pos, mesh.clone()));
        }
    }

    fn remove_chunk_object(&self, pos: ChunkPos) {
        if let Some(chunk_object_tx) = &self.chunk_object_tx {
            let _ = chunk_object_tx.send(ChunkObjectEvent::Removed(pos));
        }
    }

    fn load_chunk_if_is_empty(&mut self, pos: ChunkPos, chunks: &mut ChunkStorage) {
        let curr_chunk_state = self.world.get_chunk_state(pos);

        if curr_chunk_state == ChunkState::Empty {
            self.change_chunk_state(pos, ChunkState::Loading, chunks);
        }
    }

    fn create_chunk_status(
        &self,
        pos: ChunkPos,
        chunks: &ChunkStorage,
        controller_pos: &ControllerPos,
    ) -> ChunkStatus {
        let is_within_render =
            self.is_within_distance(pos, self.config.render_distance, controller_pos);
        let is_within_load =
            self.is_within_distance(pos, self.config.load_distance, controller_pos);
        let loaded = chunks.get_chunk(pos).is_some();
        //let mesh_built = self.chunk_builder.is_chunk_with_latest_version_built(pos);
        let mesh_built = false;
        let needs_rebuild = self.world.get_chunk_need_rebuild(pos);

        ChunkStatus {
            is_within_render,
            is_within_load,
            loaded,
            mesh_built,
            needs_rebuild,
        }
    }
}

impl fmt::Debug for ChunkManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChunkManager")
            .field("world", &self.world)
            // .field("chunk_builder", &self.chunk_builder)
            .field("chunk_loader", &self.chunk_loader)
            .finish()
    }
}
