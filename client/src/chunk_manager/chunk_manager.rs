use shared::{chunk_loader::chunk_loader, entities::{Chunk, ChunkPos, CHUNK_SIZE}};
use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

use crate::chunk_builder::{ChunkMesh};

use super::physical_world::PhysicalWorld;

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
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk);
    fn get_builded_chunks(&mut self) -> HashMap<ChunkPos, ChunkMesh>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    load_distance: usize,
    render_distance: usize,
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
    last_controller_pos: Option<ChunkPos>,
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
            last_controller_pos: None,
        }
    }

    pub fn set_chunk_object_callback(&mut self, callback: Arc<Mutex<dyn ChunkObjectCallback>>) {
        self.chunk_object_callback = Some(callback);
    }

    pub fn set_event_callback(&mut self, callback: Arc<Mutex<dyn EventCallback>>) {
        self.event_callback = Some(callback);
    }

    pub fn update_controller_pos(&mut self, controller_pos: ChunkPos) {
        if let Some(last_controller_pos) = self.last_controller_pos {
            if controller_pos == last_controller_pos {
                return;
            }
        }

        self.update_chunk_states_in_controller_range(controller_pos);
        self.last_controller_pos = Some(controller_pos);
    }

    pub fn check_builded_chunks(&mut self) {
        for (pos, mesh) in self.chunk_builder.get_builded_chunks() {
            self.add_drawn_chunk(pos, mesh);
        }
    }

    pub fn get_world(&self) -> &PhysicalWorld {
        &self.world
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.world.world.get_chunk(pos)
    }

    pub fn get_or_load_chunk(&mut self, pos: ChunkPos) -> &Chunk {
        let curr_chunk_state = self.world.get_chunk_state(pos);

        if curr_chunk_state == None {
            let chunk = self.chunk_loader.load_chunk(pos);
            self.change_world_chunk(pos, chunk);

            self.world.change_chunk_state(pos, super::ChunkState::Loaded);
        }

        self.get_chunk(pos).unwrap()
    }

    pub fn set_chunk(&mut self, pos: ChunkPos, new_chunk: Chunk) {
        let curr_chunk_state = self.world.get_chunk_state(pos);

        if curr_chunk_state == None {
            self.world.change_chunk_state(pos, super::ChunkState::Loaded);
        } 
        else if curr_chunk_state == Some(&super::ChunkState::Drawn) {
            self.world.change_chunk_state(pos, super::ChunkState::ToDraw);
            self.chunk_builder.build_chunk(pos, &new_chunk);
        }

        self.change_world_chunk(pos, new_chunk);

        // Co jeśli stan był już to_draw? 
        // Co jeśli wtedy narysowanie nowej wersji zajmie mniej czasu, niż narysowanie wersji starej? 
        // Odp. Gra wyrenderuje mesh starej wersji chunka, ale dane będą nowej wersji. - czyli mamy problem.
        // Wniosek: muszę jakoś anulować poprzednie tworzenie mesha, albo je zignoro
        // uuid?
    }

    fn add_drawn_chunk(&mut self, pos: ChunkPos, chunk_mesh: ChunkMesh) {
        // to_draw -> drawn
        assert_eq!(self.world.get_chunk_state(pos), Some(&super::ChunkState::ToDraw), "Drawn chunk must first be in to_draw state.");

        self.world.add_chunk_mesh(pos, chunk_mesh.clone());
        self.world.change_chunk_state(pos, super::ChunkState::Drawn);

        if let Some(callback) = &self.chunk_object_callback {
            if let Ok(mut callback) = callback.lock() {
                callback.chunk_object_created(pos, &chunk_mesh);
            }
        }
    }

    fn update_chunk_states_in_controller_range(&mut self, controller_pos: ChunkPos) {
        // empty -> loaded
        let mut chunks_in_load_distance = HashSet::<ChunkPos>::default();

        Self::for_each_chunk_in_distance(controller_pos, self.config.load_distance, self.config.dynamic_vertical_loading, |pos| {
            chunks_in_load_distance.insert(pos);
            let curr_chunk_state = self.world.get_chunk_state(pos);

            if curr_chunk_state == None {
                let chunk = self.chunk_loader.load_chunk(pos);
                self.change_world_chunk(pos, chunk);

                self.world.change_chunk_state(pos, super::ChunkState::Loaded);
            }
        });

        // loaded -> to_draw
        let mut chunks_in_render_distance = HashSet::<ChunkPos>::default();

        Self::for_each_chunk_in_distance(controller_pos, self.config.render_distance, self.config.dynamic_vertical_loading, |pos| {
            chunks_in_render_distance.insert(pos);
            let curr_chunk_state = self.world.get_chunk_state(pos);

            if curr_chunk_state == Some(&super::ChunkState::Loaded) {
                if let Some(chunk_to_build) = self.world.world.get_chunk(pos) {
                    self.chunk_builder.build_chunk(pos, chunk_to_build);
                    self.world.change_chunk_state(pos, super::ChunkState::ToDraw);
                } else {
                    eprintln!("Chunk to build don't exist in world!");
                }
            }
        });

        // drawn -> loaded
        let drawn_chunks: HashSet<ChunkPos> = self.get_chunks_with_state(super::ChunkState::Drawn);

        for pos in drawn_chunks {
            if !chunks_in_render_distance.contains(&pos) {
                self.world.chunk_meshes.remove(&pos);
                self.world.change_chunk_state(pos, super::ChunkState::Loaded);

                if let Some(callback) = &self.chunk_object_callback {
                    if let Ok(mut callback) = callback.lock() {
                        callback.chunk_object_removed(pos);
                    }
                }
            }
        }

        // loaded -> empty
        let loaded_chunks: HashSet<ChunkPos> = self.get_chunks_with_state(super::ChunkState::Loaded);

        for pos in loaded_chunks {
            if !chunks_in_load_distance.contains(&pos) {
                self.world.chunk_states.remove(&pos);
                self.remove_world_chunk(pos);
            }
        }

        // TODO
        // to_draw -> loaded
    }

    fn get_chunks_with_state<T: FromIterator<ChunkPos>>(&self, state: super::ChunkState) -> T {
        self.world.chunk_states
            .iter()
            .filter(|(_, chunk_state)| **chunk_state == state)
            .map(|(chunk_pos, _)| *chunk_pos)
            .collect()
    }

    fn for_each_chunk_in_distance<F: FnMut(ChunkPos)>(controller_pos: ChunkPos, dist: usize, vertical: bool, mut func: F) {
        let controller_pos = *controller_pos / 16;

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

    fn change_world_chunk(&mut self, pos: ChunkPos, new_chunk: Chunk) {
        self.world.world.add_chunk(pos, new_chunk.clone());

        if let Some(callback) = &self.event_callback {
            if let Ok(mut callback) = callback.lock() {
                callback.chunk_update_callback(WorldChunkUpdate { chunk_pos: pos, chunk: new_chunk });
            }
        }
    }

    fn remove_world_chunk(&mut self, pos: ChunkPos) {
        if let Some(chunk) = self.world.world.get_chunk(pos) {
            if let Some(callback) = &self.event_callback {
                if let Ok(mut callback) = callback.lock() {
                    callback.chunk_update_callback(WorldChunkUpdate { chunk_pos: pos, chunk: chunk.clone() });
                }
            }
        }

        self.world.world.remove_chunk(pos);
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_for_each_chunk_in_distance() {
        let mut actual_positions: HashSet<ChunkPos> = HashSet::new();

        ChunkManager::for_each_chunk_in_distance(ChunkPos::new(0, 0, 0), 2, false, |pos| {
            actual_positions.insert(pos);
        });

        let expected_positions: HashSet<ChunkPos> = (-2..=2)
            .flat_map(|y| (-2..=2).map(move |x| ChunkPos::new(x * CHUNK_SIZE as isize, y * CHUNK_SIZE as isize, 0)))
            .collect();

        assert_eq!(actual_positions, expected_positions);
    }
}