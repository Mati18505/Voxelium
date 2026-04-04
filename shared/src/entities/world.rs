use super::{
    chunk::Chunk,
    types::{BlockID, BlockPos, ChunkPos},
    BlockInChunkPos,
};

pub trait ChunkRepository {
    fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk>;
    fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk>;
}

/// Returns block from repository, if it exists.
/// If it doesn't exists returns `BlockID::default()` I.e. air.
pub fn get_block(world_pos: BlockPos, chunk_repository: &impl ChunkRepository) -> BlockID {
    let chunk_pos = ChunkPos::from(world_pos);
    let in_chunk_pos = BlockInChunkPos::from(world_pos);

    chunk_repository
        .get_chunk(chunk_pos)
        .map_or(BlockID::default(), |chunk| {
            chunk.get_block_storage().get_block(in_chunk_pos)
        })
}

#[cfg(test)]
mod test {
    use crate::entities::{BlockStorage, CHUNK_SIZE};

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
        let chunk_storage =
            DummyChunkStorage(HashMap::from([(ChunkPos::new(16, 0, 0), Chunk::default())]));
        let pos = BlockPos::new(0, 0, 0);

        assert_eq!(get_block(pos, &chunk_storage), 0,);
    }

    #[test]
    fn test_get_block_existing_chunk() {
        let block_storage = BlockStorage::new(vec![3; CHUNK_SIZE.pow(3)]);
        let chunk_storage = DummyChunkStorage(HashMap::from([(
            ChunkPos::new(0, 0, 0),
            Chunk::new(block_storage),
        )]));
        let pos = BlockPos::new(0, 0, 0);

        assert_eq!(get_block(pos, &chunk_storage), 3);
    }
}
