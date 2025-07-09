use shared::entities::{BlockID, BlockInChunkPos, BlockPos, Chunk, ChunkPos, ChunkRepository};

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