use std::{error::Error, fmt};

use super::{
    chunk::Chunk,
    types::{BlockID, BlockPos, ChunkPos},
    BlockInChunkPos,
};

pub trait ChunkRepository {
    fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk>;
    fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk>;
}

pub fn get_block(
    world_pos: BlockPos,
    chunk_repository: &impl ChunkRepository,
) -> Result<BlockID, GetBlockErr> {
    let chunk_pos = ChunkPos::from(world_pos);
    let in_chunk_pos = BlockInChunkPos::from(world_pos);
    let chunk = chunk_repository
        .get_chunk(chunk_pos)
        .ok_or(GetBlockErr::OutsideOfWorld)?;

    Ok(chunk.get_block_storage().get_block(in_chunk_pos))
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
    use std::collections::hash_map::HashMap;

    #[derive(Default)]
    struct DummyChunkStorage(HashMap<ChunkPos, Chunk>);
    impl ChunkRepository for DummyChunkStorage {
        fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
            self.0.get(&pos)
        }

        fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
            self.0.get_mut(&pos)
        }
    }

    #[test]
    fn test_get_block_outside_of_world() {
        let chunk_storage = DummyChunkStorage::default();
        let pos = BlockPos::new(0, 0, 0);
        assert_eq!(
            get_block(pos, &chunk_storage),
            Err(GetBlockErr::OutsideOfWorld)
        );
    }

    #[test]
    fn test_get_block_existing_chunk() {
        let chunk_storage =
            DummyChunkStorage(HashMap::from([(ChunkPos::new(0, 0, 0), Chunk::default())]));
        let pos = BlockPos::new(0, 0, 0);

        assert_eq!(get_block(pos, &chunk_storage), Ok(0));
    }
}
