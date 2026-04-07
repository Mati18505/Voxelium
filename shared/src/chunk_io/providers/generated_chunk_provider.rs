use crate::chunk_io::providers::provider::ChunkProvider;
use crate::entities::{BlockRegistry, Chunk, ChunkPos};

use super::terrain_generator::TerrainGenerator;

#[derive(Debug, Clone)]
pub struct GeneratedChunkProvider {}

impl GeneratedChunkProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl ChunkProvider for GeneratedChunkProvider {
    fn load_chunk(&mut self, pos: ChunkPos, registry: &dyn BlockRegistry) -> Chunk {
        let mut terrain_generator = TerrainGenerator::new();
        let block_storage = terrain_generator.generate_terrain(pos, registry);

        Chunk::new(block_storage)
    }
}

impl Default for GeneratedChunkProvider {
    fn default() -> Self {
        Self::new()
    }
}
