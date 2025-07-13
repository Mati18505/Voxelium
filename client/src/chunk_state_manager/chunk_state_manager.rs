use bevy::log;
use shared::{chunk_loader::chunk_loader, entities::{Chunk, ChunkPos, ChunkRepository, CHUNK_SIZE}};
use std::fmt;

use crate::{chunk_mesh_builder::ChunkMesh, chunk_state_manager::{chunk_state, ChunkState, ChunkStatus, ChunkTransition}};

use super::{physical_world::{PhysicalWorld, Version}};

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

pub trait ChunkBuilder: Send + Sync + fmt::Debug {
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, version: Version);
    fn collect_finished_results(&mut self);
    fn take_built_chunk_mesh_by_version(&mut self, chunk_pos: ChunkPos, version: Version) -> Option<ChunkMesh>;
    fn is_chunk_mesh_built_with_version(&self, chunk_pos: ChunkPos, version: Version) -> bool;
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

/// Manage chunks dependent on controller position.
pub struct ChunkManager {
    world: PhysicalWorld,
    chunk_loader: chunk_loader::ChunkLoader,
    chunk_builder: Box<dyn ChunkBuilder>,
    chunk_object_tx: Option<crossbeam_channel::Sender<ChunkObjectEvent>>,
    event_tx: Option<crossbeam_channel::Sender<WorldChunkUpdate>>,
    config: Config,
    controller_pos: ChunkPos,
}

impl ChunkManager {
    pub fn new(chunk_loader: chunk_loader::ChunkLoader, chunk_builder: Box<dyn ChunkBuilder>, config: Config) -> Self {
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
    pub fn set_chunk_object_tx(&mut self, callback: Option<crossbeam_channel::Sender<ChunkObjectEvent>>) {
        self.chunk_object_tx = callback;
    }

    /// Sets the callback used after chunk has been modified.
    pub fn set_event_tx(&mut self, callback: Option<crossbeam_channel::Sender<WorldChunkUpdate>>) {
        self.event_tx = callback;
    }

    /// Updates the controller position and triggers chunk state updates if position has changed.
    pub fn update_controller_pos(&mut self, new_controller_pos: ChunkPos) {
        if new_controller_pos != self.controller_pos {
            self.controller_pos = new_controller_pos;
            self.update_chunk_states_in_world();
        }
    }

    /// Checks and processes chunks ready to be drawn.
    /// Should be called once per frame.
    pub fn check_built_chunks(&mut self) {
        self.chunk_builder.collect_finished_results();

        let chunks_to_draw: Vec<ChunkPos> = self.world.get_chunks_with_state(ChunkState::ToDraw);

        for chunk_pos in chunks_to_draw {
            self.update_chunk_state(chunk_pos);
        }
    }

    pub fn get_world(&self) -> &PhysicalWorld {
        &self.world
    }

    /// Gets chunk from the world or loads it if it is not loaded yet.
    /// Returns None only if the position is outside the world scope.
    pub fn get_or_load_chunk(&mut self, pos: ChunkPos) -> Option<&Chunk> {
        if !self.is_in_world_scope(pos) {
            return None;
        }
        
        self.load_chunk_if_is_empty(pos);

        self.get_chunk(pos)
    }

    fn update_chunk_states_in_world(&mut self) {
        // Load missing chunks within the load distance.
        Self::visit_chunks_in_distance(self.controller_pos, self.config.load_distance, self.config.dynamic_vertical_loading, |pos| {
            self.load_chunk_if_is_empty(pos);
        });

        // Update all existing chunks in the world.
        let chunks_in_world: Vec<ChunkPos> = self.world.chunk_states.keys().copied().collect();

        for pos in chunks_in_world {
            self.update_chunk_state(pos);
        }

        // Remove all chunks that are still empty.
        let empty_chunks_in_world: Vec<ChunkPos> = self.world.get_chunks_with_state(ChunkState::Empty);

        for pos in empty_chunks_in_world {
            self.world.remove_chunk(pos);
        }
    }

    fn visit_chunks_in_distance<F: FnMut(ChunkPos)>(controller_pos: ChunkPos, dist: usize, vertical: bool, mut func: F) {
        let controller_pos = *controller_pos / CHUNK_SIZE as isize;

        let z_start = controller_pos.z - dist as isize;
        let z_end = controller_pos.z + dist as isize;
        let y_start = controller_pos.y - dist as isize;
        let y_end = controller_pos.y + dist as isize;
        let x_start = controller_pos.x - dist as isize;
        let x_end = controller_pos.x + dist as isize;

        if vertical {
            for z in z_start..=z_end {
                for y in y_start..=y_end {
                    for x in x_start..=x_end {
                        let pos = ChunkPos::new(x * CHUNK_SIZE as isize, y * CHUNK_SIZE as isize, z * CHUNK_SIZE as isize);

                        func(pos)
                    }
                } 
            }
        } else {
            for y in y_start..=y_end {
                for x in x_start..=x_end {
                    let pos = ChunkPos::new(x * CHUNK_SIZE as isize, y * CHUNK_SIZE as isize, 0);

                    func(pos)
                }
            } 
        }
    }

    fn is_in_world_scope(&self, pos: ChunkPos) -> bool {
        if !self.config.dynamic_vertical_loading {
            if pos.z != 0 {
                return false;
            }
        }

        true
    }

    /// Always use this instead of set_chunk_state directly – handles transitions.
    fn change_chunk_state(&mut self, pos: ChunkPos, new_state: ChunkState) {
        let prev_state = self.world.get_chunk_state(pos);

        if prev_state != new_state {
            let transition = chunk_state::get_chunk_transition(prev_state, new_state);

            assert!(transition.is_some(), "Unsupported transition in chunk {:?}: {:?} -> {:?}", pos, prev_state, new_state);

            self.apply_transition(pos, transition.unwrap());
            self.world.set_chunk_state(pos, new_state);
        }
    }

    fn update_chunk_state(&mut self, pos: ChunkPos) {
        const MAX_ITERATIONS: u32 = 16;
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
                log::warn!("Chunk {:?} failed to stabilize state after {} iterations", pos, MAX_ITERATIONS);
                break;
            }

            prev_state = next_state;
            iterations += 1;
        } 
    }

    fn is_within_distance(&self, pos: ChunkPos, distance_in_chunks: usize) -> bool {
        match self.config.dynamic_vertical_loading {
            true => pos.is_within_distance(self.controller_pos, distance_in_chunks),
            false => pos.is_within_distance_2d(self.controller_pos, distance_in_chunks),
        }
    }

    fn apply_transition(&mut self, pos: ChunkPos, transition: ChunkTransition) {
        use ChunkTransition::*;

        match transition {
            EmptyToLoaded => {
                let chunk = self.chunk_loader.load_chunk(pos);

                self.world.set_chunk(pos, chunk);
            },
            LoadedToEmpty => {
                self.world.world.remove_chunk(pos);
            },
            LoadedToToDraw => {
                self.pass_chunk_to_builder(pos);
            },
            ToDrawToLoaded => {
                // TODO: Remove mesh from chunk builder.
            }
            ToDrawToDrawn => {
                let version = self.world.get_chunk_mesh_version(pos);
                let mesh = self.chunk_builder.take_built_chunk_mesh_by_version(pos, version).expect("ChunkState is drawn, but mesh is not built.");

                self.world.add_chunk_mesh(pos, mesh.clone());

                self.create_chunk_object(pos, &mesh);
            },
            DrawnToToDraw => {
                self.pass_chunk_to_builder(pos);
            },
            DrawnToLoaded => {
                self.world.chunk_meshes.remove(&pos);
                self.remove_chunk_object(pos);
            },
        }
    }

    fn pass_chunk_to_builder(&mut self, pos: ChunkPos) {
        let new_mesh_version = self.world.increment_chunk_mesh_version(pos);
        //println!("Passing chunk to builder: {:?}, version: {}", pos, new_mesh_version);

        let chunk = self.world.get_chunk(pos)
            .expect("Chunk is passed to builder, but it is not loaded.");

        self.chunk_builder.build_chunk(pos, chunk, new_mesh_version);
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

    fn load_chunk_if_is_empty(&mut self, pos: ChunkPos) {
        let curr_chunk_state = self.world.get_chunk_state(pos);

        if curr_chunk_state == ChunkState::Empty {
            self.change_chunk_state(pos, ChunkState::Loaded);
        }
    }

    // Redraws chunk only if it was drawn or to draw.
    fn redraw_chunk(&mut self, pos: ChunkPos) {
        let curr_chunk_state: ChunkState = self.world.get_chunk_state(pos);

        match curr_chunk_state {
            ChunkState::ToDraw => {
                self.pass_chunk_to_builder(pos);
            },
            ChunkState::Drawn => {
                self.change_chunk_state(pos, ChunkState::ToDraw);
            },
            _ => (),
        }   
    }
    
    fn create_chunk_status(&self, pos: ChunkPos) -> ChunkStatus {
        let is_within_render = self.is_within_distance(pos, self.config.render_distance);
        let is_within_load = self.is_within_distance(pos, self.config.load_distance);
        let mesh_version = self.world.get_chunk_mesh_version(pos);
        let mesh_built = self.chunk_builder.is_chunk_mesh_built_with_version(pos, mesh_version);

        ChunkStatus {
            is_within_render,
            is_within_load,
            mesh_built,
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

        self.redraw_chunk(pos);

        self.emit_event(WorldChunkUpdate { chunk_pos: pos, chunk: new_chunk });
    }

    /// Unloads chunk and removes it from world.
    fn remove_chunk(&mut self, pos: ChunkPos) {
        if let Some(chunk) = self.world.get_chunk(pos) {
            self.emit_event(WorldChunkUpdate { chunk_pos: pos, chunk: chunk.clone() });

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
            .field("chunk_builder", &self.chunk_builder)
            .finish()
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_for_each_chunk_in_distance() {
        let mut actual_positions: HashSet<ChunkPos> = HashSet::new();

        ChunkManager::visit_chunks_in_distance(ChunkPos::new(0, 0, 0), 2, false, |pos| {
            actual_positions.insert(pos);
        });

        let expected_positions: HashSet<ChunkPos> = (-2..=2)
            .flat_map(|y| (-2..=2).map(move |x| ChunkPos::new(x * CHUNK_SIZE as isize, y * CHUNK_SIZE as isize, 0)))
            .collect();

        assert_eq!(actual_positions, expected_positions);
    }
}