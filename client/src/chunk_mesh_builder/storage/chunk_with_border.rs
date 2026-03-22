use shared::entities::{block_in_chunk_pos_generator::BlockInChunkPosGenerator, BlockInChunkPos, BlockStorage, Chunk, CHUNK_SIZE};

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkWithBorder {
    block_storage: BlockStorage,
}

impl ChunkWithBorder {
    pub fn new(block_storage: BlockStorage) -> Self {
        assert!(block_storage.iter().len() == (CHUNK_SIZE + 2).pow(3));

        ChunkWithBorder { block_storage }
    }

    pub fn get_block_storage(&self) -> &BlockStorage {
        &self.block_storage
    }
}

impl Default for ChunkWithBorder {
    fn default() -> Self {
        let block_storage = BlockStorage::new(vec![0; (CHUNK_SIZE + 2).pow(3)]);
        Self::new(block_storage)
    }
}

pub struct ChunkWithNeighbors {
    pub chunk: Chunk,
    pub neighbors: [Chunk; 6],
}

impl Into<ChunkWithBorder> for ChunkWithNeighbors {
    fn into(self) -> ChunkWithBorder {
        let size = CHUNK_SIZE + 2;
        let mut data = vec![0; size.pow(3)];

        let idx = |x: usize, y: usize, z: usize| -> usize {
            x + size * (y + size * z)
        };

        // original chunk
        for source_pos in BlockInChunkPosGenerator::new() {
            data[idx(source_pos.x + 1, source_pos.y + 1, source_pos.z + 1)] =
                self.chunk.get_block_storage().get_block(source_pos);
        }

        // neighbors
        let [nz_pos, nz_neg, nx_pos, nx_neg, ny_pos, ny_neg] = self.neighbors;

        // -X
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let source_pos = BlockInChunkPos::new(CHUNK_SIZE - 1, y, z);
                data[idx(0, y + 1, z + 1)] =
                    nx_neg.get_block_storage().get_block(source_pos);
            }
        }

        // +X
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let source_pos = BlockInChunkPos::new(0, y, z);
                data[idx(CHUNK_SIZE + 1, y + 1, z + 1)] =
                    nx_pos.get_block_storage().get_block(source_pos);
            }
        }

        // -Y
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let source_pos = BlockInChunkPos::new(x, CHUNK_SIZE - 1, z);
                data[idx(x + 1, 0, z + 1)] =
                    ny_neg.get_block_storage().get_block(source_pos);
            }
        }

        // +Y
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let source_pos = BlockInChunkPos::new(x, 0, z);
                data[idx(x + 1, CHUNK_SIZE + 1, z + 1)] =
                    ny_pos.get_block_storage().get_block(source_pos);
            }
        }

        // -Z
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let source_pos = BlockInChunkPos::new(x, y, CHUNK_SIZE - 1);
                data[idx(x + 1, y + 1, 0)] =
                    nz_neg.get_block_storage().get_block(source_pos);
            }
        }

        // +Z
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let source_pos = BlockInChunkPos::new(x, y, 0);
                data[idx(x + 1, y + 1, CHUNK_SIZE + 1)] =
                    nz_pos.get_block_storage().get_block(source_pos);
            }
        }

        let block_storage = BlockStorage::new(data);
        ChunkWithBorder::new(block_storage)
    }
}
