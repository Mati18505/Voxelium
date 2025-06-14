

// (controller_pos, chunk_storage) -> (, )
// - load needed chunks
// - manage chunks: chunk_state

use cgmath::MetricSpace;
use shared::{chunk_loader::chunk_loader, entities::{Chunk, ChunkPos, CHUNK_SIZE}};
use std::{collections::HashSet, error::Error};

use crate::chunk_builder::{ChunkMesh};

use super::physical_world::PhysicalWorld;

pub trait ChunkBuilder {
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

    pub fn update(&mut self, controller_pos: ChunkPos) {
        if self.distance2_from_last_pos(controller_pos) <= CHUNK_SIZE * CHUNK_SIZE {
            // Offest one chunk? Move.
            self.update_chunks_on_controller_move(controller_pos);
            
        } else {
            // Completly different position? Clear, and load all.
            // TODO: Remove only chunks outside range.
            self.world = PhysicalWorld::default();
            self.update_all_chunks_in_controller_range(controller_pos);
        }
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
        let mut chunks_in_load_distance = HashSet::<ChunkPos>::default();

        Self::for_each_chunk_in_distance(controller_pos, self.config.load_distance, |pos| {
            chunks_in_load_distance.insert(pos);

            let chunk = self.chunk_loader.load_chunk(pos);
            self.world.world.add_chunk(pos, chunk);

            self.world.change_chunk_state(pos, super::ChunkState::Generated);
        });

        let mut chunks_in_render_distance = HashSet::<ChunkPos>::default();

        Self::for_each_chunk_in_distance(controller_pos, self.config.render_distance, |pos| {
            chunks_in_render_distance.insert(pos);

            self.world.change_chunk_state(pos, super::ChunkState::ToDraw);
            self.draw_chunk(pos);
        });

        for pos in chunks_in_load_distance {
            if !chunks_in_render_distance.contains(&pos) {
                self.world.change_chunk_state(pos, super::ChunkState::Empty);
                self.world.world.remove_chunk(pos);
            }
        }

        // To redraw?
        // Outside load distance?
        // chunks_to_draw -> draw_chunk
    }

    fn for_each_chunk_in_distance<F: FnMut(ChunkPos)>(controller_pos: ChunkPos, dist: usize, mut func: F) {
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

    fn update_chunks_on_controller_move(&mut self, controller_pos: ChunkPos) {
        assert!(self.last_controller_pos.is_some());

        let last_controller_pos = self.last_controller_pos.unwrap();
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