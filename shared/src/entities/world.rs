use std::{collections::HashMap, error::Error, fmt};

use super::{
    chunk::Chunk,
    types::{BlockID, BlockPos, ChunkPos},
    BlockInChunkPos,
};

pub trait ChunkRepository {
    fn set_chunk(&mut self, pos: ChunkPos, new_chunk: Chunk);
    fn remove_chunk(&mut self, pos: ChunkPos);
    fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk>;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct World {
    pub chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new() -> World {
        World {
            chunks: HashMap::new(),
        }
    }

    pub fn get_block(&self, world_pos: BlockPos) -> Result<BlockID, GetBlockErr> {
        let chunk_pos = ChunkPos::from(world_pos);
        let in_chunk_pos = BlockInChunkPos::from(world_pos);

        Ok(self
            .get_chunk(chunk_pos)
            .ok_or(GetBlockErr::OutsideOfWorld)?
            .get_block_storage()
            .get_block(in_chunk_pos))
    }
}

impl ChunkRepository for World {
    fn set_chunk(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }
    
    fn remove_chunk(&mut self, pos: ChunkPos) {
        self.chunks.remove(&pos);
    }

    fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }
}

#[derive(Debug, PartialEq)]
pub enum GetBlockErr {
    OutsideOfWorld,
}

impl Error for GetBlockErr {}

impl fmt::Display for GetBlockErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            GetBlockErr::OutsideOfWorld => "Block is outside of the world.",
        };

        write!(f, "{s}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_block_outside_of_world() {
        let world = World::new();

        let pos = BlockPos::new(0, 0, 0);
        assert_eq!(world.get_block(pos), Err(GetBlockErr::OutsideOfWorld));
    }

    #[test]
    fn test_add_chunk_and_get_block() {
        let mut world = World::new();

        world.set_chunk(ChunkPos::new(0, 0, 0), Chunk::default());

        let pos = BlockPos::new(0, 0, 0);
        assert_eq!(world.get_block(pos), Ok(0));
    }
}
