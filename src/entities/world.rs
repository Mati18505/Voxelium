use std::{collections::HashMap, error::Error, fmt};

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
    pub fn get_block(&self, world_pos: BlockPos) -> Result<BlockID, GetBlockErr> {
        if world_pos.z < 0 {
            return Err(GetBlockErr::OutsideOfWorld);
        }

        let chunk_pos = ChunkPos::from(world_pos);
        let in_chunk_pos = BlockInChunkPos::from(world_pos);

        Ok(self.get_chunk(chunk_pos).ok_or(GetBlockErr::OutsideOfWorld)?.get_block_storage().get_block(in_chunk_pos))
    }
}

#[derive(Debug, PartialEq)]
pub enum GetBlockErr {
    OutsideOfWorld
}

impl Error for GetBlockErr {}

impl fmt::Display for GetBlockErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            GetBlockErr::OutsideOfWorld => "Block is outside of the world.",
        };

        write!(f, "{}", s)
    }
}
