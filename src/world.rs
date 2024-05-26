use crate::chunk::Chunk;
use crate::types::{ChunkPos, BlockPos, BlockID, CHUNK_SIZE};
use cgmath::Vector3;

use std::collections::HashMap;

pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new() -> World {
        World{
            chunks: HashMap::new()
        }
    }
    pub fn get_chunk_cloned(&self, pos: &ChunkPos) -> Option<Chunk> {
        self.chunks.get(&pos).map(|c| c.clone())
    }
    pub fn get_chunk(&self, pos: &ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }
    pub fn add_chunk(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }
    pub fn get_block(&self, world_pos: &BlockPos) -> Result<BlockID, &str> {
        let chunk_pos = ChunkPos::from(world_pos);
        let block_in_chunk_pos = BlockPos(world_pos.0 - chunk_pos.0);
        let block_in_chunk_pos = block_in_chunk_pos.0.cast().unwrap();
/*
        if block_in_chunk_pos.x < 0 || block_in_chunk_pos.y < 0 || block_in_chunk_pos.z < 0 ||
        block_in_chunk_pos.x >= CHUNK_SIZE as isize || block_in_chunk_pos.y >= CHUNK_SIZE as isize || block_in_chunk_pos.z >= CHUNK_SIZE as isize {
            return Err("Outside of world");
        }*/

        Ok(self.get_chunk(&chunk_pos).ok_or("Block outside of the world")?.get_block_storage().get_block(block_in_chunk_pos))
    }
    pub fn ray_cast(start: Vector3<isize>, dir: Vector3<isize>, range: f32) -> RayCastResult {
        todo!()
    }
}

struct RayCastResult {
    hitted: bool,
    hit_point: Vector3<f32>,
    previous_hit_point: f32,
}

impl RayCastResult {
    fn new() -> Self {
        RayCastResult {
            hitted: false,
            hit_point: Vector3::new(0.0, 0.0, 0.0),
            previous_hit_point: 0.0,
        }
    }
}