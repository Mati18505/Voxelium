use crate::chunk_io;
use crate::entities::{Chunk, ChunkPos};

use super::terrain_generator::TerrainGenerator;

#[derive(Debug, Clone)]
pub struct GeneratedChunkProvider {}

impl GeneratedChunkProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl chunk_io::ChunkProvider for GeneratedChunkProvider {
    fn load_chunk(&mut self, pos: ChunkPos) -> Chunk {
        let mut terrain_generator = TerrainGenerator::new();
        let block_storage = terrain_generator.generate_terrain(pos);

        Chunk::new(block_storage)
    }
}

impl Default for GeneratedChunkProvider {
    fn default() -> Self {
        Self::new()
    }
}
