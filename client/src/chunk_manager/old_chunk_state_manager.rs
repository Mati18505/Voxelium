use bevy::log;
use shared::{
    chunk_io::chunk_loader,
    entities::{Chunk, ChunkPos, ChunkRepository, CHUNK_SIZE},
};
use std::fmt;

use crate::chunk_mesh_builder::{
    builders::{ChunkBuilder, Versioned},
    ChunkMesh,
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
    chunk_builder: Box<dyn VersionedChunkBuilder<()>>,
    chunk_object_tx: Option<crossbeam_channel::Sender<ChunkObjectEvent>>,
    event_tx: Option<crossbeam_channel::Sender<WorldChunkUpdate>>,
    config: Config,
    controller_pos: ChunkPos,
}

impl ChunkManager {
    pub fn new(
        chunk_loader: chunk_loader::ChunkLoader,
        chunk_builder: Box<dyn VersionedChunkBuilder<()>>,
        config: Config,
    ) -> Self {
        ChunkManager {
            world: PhysicalWorld::default(),
            chunk_loader,
            chunk_builder,
            chunk_object_tx: None,
            event_tx: None,
            config,
            controller_pos: ChunkPos::new(0, 0, 0),
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

    /// Gets chunk from the world or loads it if it is not loaded yet.
    /// Returns None only if the position is outside the world scope.
    #[allow(dead_code)]
    pub fn get_or_load_chunk(&mut self, pos: ChunkPos) -> Option<&Chunk> {
        if !self.is_in_world_scope(pos) {
            return None;
        }

        self.load_chunk_if_is_empty(pos);

        self.get_chunk(pos)
    }

    fn is_in_world_scope(&self, pos: ChunkPos) -> bool {
        if !self.config.dynamic_vertical_loading && pos.z != 0 {
            return false;
        }

        true
    }

    fn apply_transition(&mut self, pos: ChunkPos, transition: ChunkTransition) {
        use ChunkTransition::*;

        match transition {
            EmptyToLoading => {
                // send ChunkLoadRequest::load
                self.chunk_loader.load_chunk(pos);
            }
            LoadingToEmpty => {
                // send ChunkLoadRequest::cancel
                self.chunk_loader.cancel_loading_chunk(pos);
            }
            LoadingToLoaded => {
                log::debug!("Loaded chunk {:?}", pos);
            }
            LoadedToEmpty => {
                self.world.world.remove_chunk(pos);
            }
            LoadedToToDraw => {
                self.pass_chunk_to_builder(pos);
            }
            ToDrawToLoaded => {
                // send ChunkBuildRequest::cancel
                // TODO: Remove mesh from chunk builder.
                // self.chunk_builder.remove_chunk(chunk_pos);
            }
            ToDrawToDrawn => {
                let mesh = self
                    .chunk_builder
                    .take_chunk_built_with_latest_version(pos)
                    .expect("ChunkState is drawn, but mesh is not built.")
                    .0;

                self.world.add_chunk_mesh(pos, mesh.clone());

                // send ChunkEntityRequest::create
                self.create_chunk_object(pos, &mesh);
            }
            DrawnToToDraw => {
                self.pass_chunk_to_builder(pos);
            }
            DrawnToLoaded => {
                self.world.chunk_meshes.remove(&pos);
                // send ChunkEntityRequest::remove
                self.remove_chunk_object(pos);
            }
        }
    }

    fn pass_chunk_to_builder(&mut self, pos: ChunkPos) {
        let chunk = self
            .world
            .get_chunk(pos)
            .expect("Chunk is passed to builder, but it is not loaded.");

        // send ChunkBuildRequest::build
        self.chunk_builder.force_build(pos, chunk, ());
        self.world.remove_chunk_need_rebuild(pos);
    }

    fn load_chunk_if_is_empty(&mut self, pos: ChunkPos) {
        let curr_chunk_state = self.world.get_chunk_state(pos);

        if curr_chunk_state == ChunkState::Empty {
            self.change_chunk_state(pos, ChunkState::Loading);
        }
    }


    /// Always use this instead of set_chunk_state directly – handles transitions.
    fn change_chunk_state(&mut self, pos: ChunkPos, new_state: ChunkState) {
        let prev_state = self.world.get_chunk_state(pos);

        if prev_state != new_state {
            let transition = chunk_state::get_chunk_transition(prev_state, new_state);

            assert!(
                transition.is_some(),
                "Unsupported transition in chunk {pos:?}: {prev_state:?} -> {new_state:?}"
            );

            self.apply_transition(pos, transition.unwrap());
            self.world.set_chunk_state(pos, new_state);
        }
    }

    const MAX_ITERATIONS: u32 = 16;

    #[derive(Debug, Error, Clone, PartialEq)]
    pub enum ChunkStateUpdateError {
        #[error("Chunk {:?} failed to stabilize state after {MAX_ITERATIONS} iterations")]
        MaxIterationsExceeded(ChunkPos),
    }

    fn update_chunk_state(curr_state: ChunkState, status: ChunkStatus, pos: ChunkPos) -> Result<ChunkState, ChunkStateUpdateError>  {
        let mut iterations = 1;
        let mut prev_state = self.world.get_chunk_state(pos);

        loop {
            let chunk_status = self.create_chunk_status(pos);
            let next_state = chunk_state::get_next_chunk_state(prev_state, chunk_status);

            if next_state == prev_state {
                break;
            }

            self.change_chunk_state(pos, next_state);
            log::trace!("Chunk {:?}: {:?} -> {:?}", pos, prev_state, next_state);

            if iterations >= MAX_ITERATIONS {
                return Err(ChunkStateUpdateError::UnknownBlockType(pos))
                break;
            }

            prev_state = next_state;
            iterations += 1;
        }
}

}

// public API
impl ChunkRepository for ChunkManager {
    /// Sets and redraws chunk without loading it.
    fn set_chunk(&mut self, pos: ChunkPos, new_chunk: Chunk) {
        self.world.set_chunk(pos, new_chunk.clone());

        let curr_chunk_state: ChunkState = self.world.get_chunk_state(pos);

        if curr_chunk_state == ChunkState::Empty {
            self.world.set_chunk_state(pos, ChunkState::Loaded);
        }

        self.world.set_chunk_need_rebuild(pos);
        self.update_chunk_state(pos);

        self.emit_event(WorldChunkUpdate {
            chunk_pos: pos,
            chunk: new_chunk,
        });
    }

    /// Unloads chunk and removes it from world.
    fn remove_chunk(&mut self, pos: ChunkPos) {
        if let Some(chunk) = self.world.get_chunk(pos) {
            self.emit_event(WorldChunkUpdate {
                chunk_pos: pos,
                chunk: chunk.clone(),
            });

            let chunk_state = self.world.get_chunk_state(pos);

            if chunk_state == ChunkState::Drawn || chunk_state == ChunkState::ToDraw {
                self.change_chunk_state(pos, ChunkState::Loaded);
            }

            self.change_chunk_state(pos, ChunkState::Empty);
        }

        self.world.remove_chunk(pos);
    }

    fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.world.get_chunk(pos)
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
