use super::terrain_generator::TerrainGenerator;
use crate::chunk_io;
use crate::entities::{Chunk, ChunkPos};

#[derive(Debug, Clone)]
pub struct GeneratedChunkProvider {
    terrain_generator: TerrainGenerator,
}

impl GeneratedChunkProvider {
    pub fn new(terrain_generator: TerrainGenerator) -> Self {
        Self { terrain_generator }
    }
}

impl chunk_io::ChunkProvider for GeneratedChunkProvider {
    fn load_chunk(&mut self, pos: ChunkPos) -> Chunk {
        let block_storage = self.terrain_generator.generate_terrain(pos);

        Chunk::new(block_storage)
    }
}

impl Default for GeneratedChunkProvider {
    fn default() -> Self {
        Self::new(TerrainGenerator::default())
    }
}
