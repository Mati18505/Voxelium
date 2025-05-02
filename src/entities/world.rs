use std::collections::HashMap;

use super::{chunk::Chunk, types::{BlockID, BlockPos, ChunkPos}, BlockInChunkPos};

#[derive(Debug, Clone, PartialEq)]
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
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }
    pub fn get_block(&self, world_pos: BlockPos) -> Result<BlockID, &str> {
        if world_pos.z < 0 {
            return Err("Block outside of the world.");
        }

        let chunk_pos = ChunkPos::from(world_pos);
        let in_chunk_pos = BlockInChunkPos::from_world_and_chunk(world_pos, chunk_pos);

        Ok(self.get_chunk(chunk_pos).ok_or("Block outside of the world")?.get_block_storage().get_block(in_chunk_pos))
    }
}
