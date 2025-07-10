use shared::{chunk_loader::chunk_loader, entities::{Chunk, ChunkPos, ChunkRepository, CHUNK_SIZE}};
use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

use crate::{chunk_mesh_builder::ChunkMesh, chunk_state_manager::chunk_state_to_behavior};

use super::{physical_world::{PhysicalWorld, Version}, ChunkManagerContext};

pub trait ChunkObjectCallback: Send + Sync {
    fn chunk_object_created(&mut self, chunk_pos: ChunkPos, chunk_mesh: &ChunkMesh);
    fn chunk_object_removed(&mut self, chunk_pos: ChunkPos);
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldChunkUpdate {
    pub chunk_pos: ChunkPos,
    pub chunk: Chunk,
}

pub trait EventCallback: Send + Sync {
    fn chunk_update_callback(&mut self, ev: WorldChunkUpdate);
}

pub trait ChunkBuilder: Send + Sync {
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, version: Version);
    fn collect_finished_results(&mut self);
    fn take_builded_chunk_mesh(&mut self, chunk_pos: ChunkPos) -> Option<(ChunkMesh, Version)>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub load_distance: usize,
    pub render_distance: usize,
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

// Manage chunks dependent on controller position.
pub struct ChunkManager {
    world: PhysicalWorld,
    chunk_loader: chunk_loader::ChunkLoader,
    chunk_builder: Box<dyn ChunkBuilder>,
    chunk_object_callback: Option<Arc<Mutex<dyn ChunkObjectCallback>>>,
    event_callback: Option<Arc<Mutex<dyn EventCallback>>>,
    config: Config,
    last_controller_pos: ChunkPos,
}

impl ChunkManager {
    pub fn new(chunk_loader: chunk_loader::ChunkLoader, chunk_builder: Box<dyn ChunkBuilder>, config: Config) -> Self {
        ChunkManager {
            world: PhysicalWorld::default(),
            chunk_loader,
            chunk_builder,
            chunk_object_callback: None,
            event_callback: None,
            config,
            last_controller_pos: ChunkPos::new(0, 0, 0),
        }
    }

    pub fn set_chunk_object_callback(&mut self, callback: Arc<Mutex<dyn ChunkObjectCallback>>) {
        self.chunk_object_callback = Some(callback);
    }

    pub fn set_event_callback(&mut self, callback: Arc<Mutex<dyn EventCallback>>) {
        self.event_callback = Some(callback);
    }

    pub fn update_controller_pos(&mut self, controller_pos: ChunkPos) {
        if controller_pos == self.last_controller_pos {
            return;
        }

        self.update_chunk_states_in_world(controller_pos);
        self.last_controller_pos = controller_pos;
    }

    pub fn check_builded_chunks(&mut self) {
        self.chunk_builder.collect_finished_results();
    }

    pub fn get_world(&self) -> &PhysicalWorld {
        &self.world
    }

    // Returns None only if the position is outside the world scope.
    pub fn get_or_load_chunk(&mut self, pos: ChunkPos) -> Option<&Chunk> {
        if !self.is_in_world_scope(pos) {
            return None;
        }
        
        self.load_chunk_if_is_empty(pos);

        self.get_chunk(pos)
    }

    fn update_chunk_states_in_world(&mut self, controller_pos: ChunkPos) {
        Self::visit_chunks_in_distance(controller_pos, self.config.load_distance, self.config.dynamic_vertical_loading, |pos| {
            self.load_chunk_if_is_empty(pos);
            self.update_chunk_state(&pos);
        });

        let chunks_in_world: Vec<ChunkPos> = self.world.world.chunks.keys().copied().collect();

        for pos in chunks_in_world {
            if !pos.is_within_distance(controller_pos, self.config.load_distance) {
                self.remove_chunk(pos);
            }
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

    fn change_chunk_state(&mut self, pos: ChunkPos, new_state: super::ChunkState) {
        if let Some(state) = self.world.get_chunk_state(pos).copied() {
            let mut ctx = self.create_chunk_manager_context();
            let state_behavior = chunk_state_to_behavior(state);

            state_behavior.on_exit(&mut ctx, pos);
        }
        let mut ctx = self.create_chunk_manager_context();
        let new_state_behavior = chunk_state_to_behavior(new_state);

        new_state_behavior.on_enter(&mut ctx, pos);
        self.world.change_chunk_state(pos, new_state);
    }

    fn update_chunk_state(&mut self, pos: &ChunkPos) {
        let controller_pos = self.last_controller_pos.clone();

        if let Some(state) = self.world.chunk_states.get(pos).copied() {
            let state_behavior = chunk_state_to_behavior(state);
            let mut ctx: ChunkManagerContext = self.create_chunk_manager_context();
            let new_state = state_behavior.update(&mut ctx, *pos, controller_pos);

            if new_state != state {
                self.change_chunk_state(*pos, new_state);
            }
        }
    }

    fn with_event_callback<F: FnOnce(&mut dyn EventCallback)>(&self, f: F)
    {
        if let Some(cb) = &self.event_callback {
            if let Ok(mut cb) = cb.lock() {
                f(&mut *cb);
            }
        }
    }

    fn load_chunk_if_is_empty(&mut self, pos: ChunkPos) {
        let curr_chunk_state = self.world.get_chunk_state(pos);

        if curr_chunk_state == None {
            self.change_chunk_state(pos, super::ChunkState::Loaded);
        } 
    }
    fn create_chunk_manager_context(&mut self) -> ChunkManagerContext {
        ChunkManagerContext {
            chunk_loader: &mut self.chunk_loader,
            chunk_builder: &mut *self.chunk_builder,
            world: &mut self.world,
            chunk_object_callback: &self.chunk_object_callback,
            config: &self.config,
        }
    }
}

impl ChunkRepository for ChunkManager {
    fn set_chunk(&mut self, pos: ChunkPos, new_chunk: Chunk) {
        if let Some(curr_chunk_state) = self.world.get_chunk_state(pos).copied() {
            let state_behavior = chunk_state_to_behavior(curr_chunk_state);

            state_behavior.notify_chunk_modified(&mut self.create_chunk_manager_context(), pos);

            self.world.set_chunk(pos, new_chunk.clone());
            self.with_event_callback(|cb| cb.chunk_update_callback(WorldChunkUpdate { chunk_pos: pos, chunk: new_chunk }));
        } else {
            assert!(true, "set chunk needs chunk to be loaded");
        }
    }

    fn remove_chunk(&mut self, pos: ChunkPos) {
        if let Some(chunk) = self.world.get_chunk(pos) {
            self.with_event_callback(|cb| cb.chunk_update_callback(super::WorldChunkUpdate { chunk_pos: pos, chunk: chunk.clone() }));
            self.world.remove_chunk(pos);
        }
    }

    fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.world.get_chunk(pos)
    }
}


#[cfg(test)]
mod test {
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