use crate::entities::{BlockID, BlockInChunkPos, BlockPos, Chunk, ChunkPos, ChunkRepository, Prefab};

pub fn set_block_and_update_chunk<T: ChunkRepository>(
    chunk_repository: &mut T,
    set_pos: BlockPos,
    new_block: BlockID,
) {
    let chunk_pos = ChunkPos::from(set_pos);
    let block_in_chunk_pos = BlockInChunkPos::from(set_pos);

    if let Some(chunk) = chunk_repository.get_chunk(chunk_pos) {
        let mut new_block_storage = chunk.get_block_storage().clone();

        new_block_storage.set_block(block_in_chunk_pos, new_block);
        let new_chunk = Chunk::new(new_block_storage);

        chunk_repository.set_chunk(chunk_pos, new_chunk);
    }
}

pub fn instantiate_prefab<T: ChunkRepository>(
    chunk_repository: &mut T,
    prefab: Prefab,
) {
    for voxel in prefab.get_voxels() {
        let chunk_pos = ChunkPos::from(voxel.pos);

        if let None = chunk_repository.get_chunk(chunk_pos) {
            chunk_repository.set_chunk(chunk_pos, Chunk::default());
        }

        set_block_and_update_chunk(chunk_repository, voxel.pos, voxel.id);
    }
}

