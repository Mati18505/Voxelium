use bevy::ecs::query::Has;
use cgmath::MetricSpace;
use shared::{chunk_loader::chunk_loader, entities::{Chunk, ChunkPos, CHUNK_SIZE}};
use std::collections::HashSet;

use crate::chunk_builder::{ChunkMesh};

use super::physical_world::PhysicalWorld;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    load_distance: usize,
    render_distance: usize,
}

impl Config {
    pub fn new(load_distance: usize, render_distance: usize) -> Self {
        assert!(render_distance <= load_distance);

        Config {
            load_distance,
            render_distance,
        }
    }
}

// Manage chunks dependent on controller position.
pub struct ChunkManager {
    world: PhysicalWorld,
    chunk_loader: chunk_loader::ChunkLoader,
    config: Config,
    last_controller_pos: Option<ChunkPos>,
}

impl ChunkManager {
    pub fn new(chunk_loader: chunk_loader::ChunkLoader, config: Config) -> Self {
        ChunkManager {
            world: PhysicalWorld::default(),
            chunk_loader,
            config,
            last_controller_pos: None,
        }
    }

    pub fn update_controller_pos(&mut self, controller_pos: ChunkPos) -> bool {
        if let Some(last_controller_pos) = self.last_controller_pos {
            if controller_pos == last_controller_pos {
                return false;
            }
        }

        self.update_chunk_states_in_controller_range(controller_pos);
        self.last_controller_pos = Some(controller_pos);
        
        return true;
    }

    pub fn get_world(&self) -> &PhysicalWorld {
        &self.world
    }

    pub fn get_chunks_to_draw(&self) -> Vec<ChunkPos> {
        self.get_chunks_with_state(super::ChunkState::ToDraw)
    }

    pub fn add_drawn_chunk(&mut self, pos: ChunkPos, chunk_mesh: ChunkMesh) {
        assert_eq!(self.world.get_chunk_state(pos), Some(&super::ChunkState::ToDraw), "Drawn chunk must first be in to_draw state.");

        self.world.add_chunk_mesh(pos, chunk_mesh);
        self.world.change_chunk_state(pos, super::ChunkState::Drawn);
    }

    fn update_chunk_states_in_controller_range(&mut self, controller_pos: ChunkPos) {

        // empty -> generated
        let mut chunks_in_load_distance = HashSet::<ChunkPos>::default();

        Self::for_each_chunk_in_distance(controller_pos, self.config.load_distance, |pos| {
            chunks_in_load_distance.insert(pos);
            let curr_chunk_state = self.world.get_chunk_state(pos);

            if curr_chunk_state == None {
                let chunk = self.chunk_loader.load_chunk(pos);
                self.world.world.add_chunk(pos, chunk);

                self.world.change_chunk_state(pos, super::ChunkState::Generated);
            }
        });

        // generated -> to_draw
        let mut chunks_in_render_distance = HashSet::<ChunkPos>::default();

        Self::for_each_chunk_in_distance(controller_pos, self.config.render_distance, |pos| {
            chunks_in_render_distance.insert(pos);
            let curr_chunk_state = self.world.get_chunk_state(pos);

            if curr_chunk_state == Some(&super::ChunkState::Generated) {
                self.world.change_chunk_state(pos, super::ChunkState::ToDraw);
            }
        });

        // drawn -> generated
        let drawn_chunks: HashSet<ChunkPos> = self.get_chunks_with_state(super::ChunkState::Drawn);

        for pos in drawn_chunks {
            if !chunks_in_render_distance.contains(&pos) {
                self.world.chunk_meshes.remove(&pos);
                self.world.change_chunk_state(pos, super::ChunkState::Generated);
            }
        }

        // generated -> empty
        let loaded_chunks: HashSet<ChunkPos> = self.get_chunks_with_state(super::ChunkState::Generated);

        for pos in loaded_chunks {
            if !chunks_in_load_distance.contains(&pos) {
                self.world.chunk_states.remove(&pos);
                self.world.world.remove_chunk(pos);
            }
        }

        // TODO
        // to_draw -> drawn: async
        // drawn -> to_draw: redraw (chunk update)
        // to_draw -> generated
    }

    fn get_chunks_with_state<T: FromIterator<ChunkPos>>(&self, state: super::ChunkState) -> T {
        self.world.chunk_states
            .iter()
            .filter(|(_, chunk_state)| **chunk_state == state)
            .map(|(chunk_pos, _)| *chunk_pos)
            .collect()
    }

    fn for_each_chunk_in_distance<F: FnMut(ChunkPos)>(controller_pos: ChunkPos, dist: usize, mut func: F) {
        let controller_pos = *controller_pos / 16;

        let y_start = controller_pos.y - dist as isize;
        let y_end = controller_pos.y + dist as isize;
        let x_start = controller_pos.x - dist as isize;
        let x_end = controller_pos.x + dist as isize;

        for y in y_start..=y_end {
            for x in x_start..=x_end {
                let pos = ChunkPos::new(x * CHUNK_SIZE as isize, y * CHUNK_SIZE as isize, 0);

                func(pos)
            }
        } 
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_for_each_chunk_in_distance() {
        let mut actual_positions: HashSet<ChunkPos> = HashSet::new();

        ChunkManager::for_each_chunk_in_distance(ChunkPos::new(0, 0, 0), 2, |pos| {
            actual_positions.insert(pos);
        });

        let expected_positions: HashSet<ChunkPos> = (-2..=2)
            .flat_map(|y| (-2..=2).map(move |x| ChunkPos::new(x * CHUNK_SIZE as isize, y * CHUNK_SIZE as isize, 0)))
            .collect();

        assert_eq!(actual_positions, expected_positions);
    }
}