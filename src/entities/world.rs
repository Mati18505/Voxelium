use std::collections::HashMap;

use super::{chunk::Chunk, types::{BlockID, BlockPos, ChunkPos}};

pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new() -> World {
        World{
            chunks: HashMap::new()
        }
    }
    pub fn add_chunk(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }
    pub fn get_chunk(&self, pos: &ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }
    pub fn get_block(&self, world_pos: &BlockPos) -> Result<BlockID, &str> {
        let chunk_pos = ChunkPos::from(world_pos);
        let in_chunk_pos: BlockPos = BlockPos(world_pos.0 - chunk_pos.0);
        let in_chunk_pos = in_chunk_pos.0.cast().unwrap();

        Ok(self.get_chunk(&chunk_pos).ok_or("Block outside of the world")?.get_block_storage().get_block(in_chunk_pos))
    }
}
