// (controller_pos, chunk_storage) -> (, )
// - load needed chunks
// - manage chunks: chunk_state

use cgmath::{MetricSpace, Vector3};
use shared::{chunk_loader::chunk_loader, entities::{chunk, Chunk, ChunkPos, CHUNK_SIZE}};
use std::{collections::HashSet, error::Error, sync::Arc};

use crate::chunk_builder::{ChunkMesh};

use super::physical_world::PhysicalWorld;

pub trait ChunkBuilder: Send + Sync {
    fn build_chunk(&self, chunk: &Chunk) -> ChunkMesh;
}

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
    chunk_builder: Box<dyn ChunkBuilder>,
    config: Config,
    last_controller_pos: Option<ChunkPos>,
}

impl ChunkManager {
    pub fn new(chunk_loader: chunk_loader::ChunkLoader, chunk_builder: Box<dyn ChunkBuilder>, config: Config) -> Self {
        ChunkManager {
            world: PhysicalWorld::default(),
            chunk_loader,
            chunk_builder,
            config,
            last_controller_pos: None,
        }
    }

    pub fn update(&mut self, controller_pos: ChunkPos) -> bool {
        let distance = self.distance2_from_last_pos(controller_pos);

        if distance == 0 {
            return false;

        } else if distance <= CHUNK_SIZE * CHUNK_SIZE {
            // Offest one chunk? Move.
            self.update_all_chunks_in_controller_range(controller_pos);
            println!("Offset one chunk! {:?}", controller_pos);
            
        } else {
            // Completly different position? Clear, and load all.
            // TODO: Remove only chunks outside range.
            println!("Completly different position! {:?}", controller_pos);
            self.world = PhysicalWorld::default();
            self.update_all_chunks_in_controller_range(controller_pos);
        }
        self.last_controller_pos = Some(controller_pos);
        
        return true;
    }

    pub fn get_world(&self) -> &PhysicalWorld {
        &self.world
    }

    fn distance2_from_last_pos(&self, new_pos: ChunkPos) -> usize {
        if let Some(last_pos) = self.last_controller_pos {
            last_pos.distance2(*new_pos) as usize 
        } else {
            usize::MAX
        }
    }

    fn update_all_chunks_in_controller_range(&mut self, controller_pos: ChunkPos) {

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

        // generated -> to_draw (drawn)
        let mut chunks_in_render_distance = HashSet::<ChunkPos>::default();

        Self::for_each_chunk_in_distance(controller_pos, self.config.render_distance, |pos| {
            chunks_in_render_distance.insert(pos);
            let curr_chunk_state = self.world.get_chunk_state(pos);

            if curr_chunk_state == Some(&super::ChunkState::Generated) {
                self.world.change_chunk_state(pos, super::ChunkState::ToDraw);
                self.draw_chunk(pos);
            }
        });

        // drawn -> generated
        println!("  drawn -> generated");
        let drawn_chunks: HashSet::<ChunkPos> = self.world.chunk_states
            .iter()
            .filter(|(_, chunk_state)| **chunk_state == super::ChunkState::Drawn)
            .map(|(chunk_pos, _)| *chunk_pos)
            .collect();

        println!("Chunks in rd: {}", chunks_in_render_distance.len());
        println!("Drawn chunks: {}", drawn_chunks.len());

        for pos in drawn_chunks {
            if !chunks_in_render_distance.contains(&pos) {
                self.world.chunk_meshes.remove(&pos);
                self.world.change_chunk_state(pos, super::ChunkState::Generated);
            }
        }


        // generated -> empty
        println!("  generated -> empty");
        let loaded_chunks: HashSet::<ChunkPos> = self.world.chunk_states
            .iter()
            .filter(|(_, chunk_state)| **chunk_state == super::ChunkState::Generated)
            .map(|(chunk_pos, _)| *chunk_pos)
            .collect();

        println!("Chunks in ld: {}", chunks_in_load_distance.len());
        println!("Loaded chunks: {}", loaded_chunks.len());

        for pos in loaded_chunks {
            if !chunks_in_load_distance.contains(&pos) {
                self.world.chunk_states.remove(&pos);
                self.world.world.remove_chunk(pos);
            }
        }

        // to_draw -> drawn: async
        // drawn -> to_draw: redraw (chunk update)
        // to_draw -> generated
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

    fn for_each_chunk_in_row_x<F: FnMut(ChunkPos)>(controller_pos: ChunkPos, dist: usize, x: isize, mut func: F) {
        let y_start = controller_pos.y - dist as isize;
        let y_end = controller_pos.y + dist as isize;

        for y in y_start..=y_end {
            let pos = ChunkPos::new(x * CHUNK_SIZE as isize, y * CHUNK_SIZE as isize, 0);

            func(pos)
        } 
    }

    fn update_chunks_on_controller_move(&mut self, controller_pos: ChunkPos) {
        assert!(self.last_controller_pos.is_some());

        let last_controller_pos = self.last_controller_pos.unwrap();
        let diff_x = controller_pos.x - last_controller_pos.x;
        let diff_y = controller_pos.y - last_controller_pos.y;

        if diff_y == 0 {
            while diff_x > 0 {
                //for_each_chunk_in_row_x(controller_pos, self.config.load_distance)
            }
        }
    }

    fn draw_chunk(&mut self, pos: ChunkPos) {
        let chunk = self.world.world.get_chunk(pos);

        if let Some(chunk) = chunk {
            let chunk_mesh = self.chunk_builder.build_chunk(chunk);

            self.world.add_chunk_mesh(pos, chunk_mesh);
            self.world.change_chunk_state(pos, super::ChunkState::Drawn);
        } else {
            eprintln!("Not loaded chunk in render distance.");
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

    #[test]
    fn test_for_each_chunk_in_row_x() {
        let mut actual_positions: HashSet<ChunkPos> = HashSet::new();

        ChunkManager::for_each_chunk_in_row_x(ChunkPos::new(0, 0, 0), 2, -2, |pos| {
            actual_positions.insert(pos);
        });

        let expected_positions: HashSet<ChunkPos> = (-2..=2)
            .map(|y| ChunkPos::new(-2 * CHUNK_SIZE as isize, y * CHUNK_SIZE as isize, 0))
            .collect();

        assert_eq!(actual_positions, expected_positions);
    }
}