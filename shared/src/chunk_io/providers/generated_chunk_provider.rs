use crate::chunk_io::providers::provider::ChunkProvider;
use crate::entities::{BlockRegistry, Chunk, ChunkPos};

use super::terrain_generator::TerrainGenerator;

#[derive(Debug, Clone)]
pub struct GeneratedChunkProvider {
    terrain_generator: TerrainGenerator,
}

impl GeneratedChunkProvider {
    pub fn new(terrain_generator: TerrainGenerator) -> Self {
        Self { terrain_generator }
    }
}

impl ChunkProvider for GeneratedChunkProvider {
    fn load_chunk(&mut self, pos: ChunkPos, registry: &dyn BlockRegistry) -> Chunk {
        let block_storage = self.terrain_generator.generate_terrain(pos, registry);

        Chunk::new(block_storage)
    }
}

impl Default for GeneratedChunkProvider {
    fn default() -> Self {
        Self::new(TerrainGenerator::default())
    }
}
