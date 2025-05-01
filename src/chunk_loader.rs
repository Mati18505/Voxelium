use crate::terrain_generator;
use crate::entities::{Chunk, ChunkPos};

pub fn load_chunk(pos: &ChunkPos) -> Chunk {
    generate_chunk(pos)
}

fn generate_chunk(pos: &ChunkPos) -> Chunk {
    let blocks = terrain_generator::generate(pos);
    Chunk::new(blocks)
}