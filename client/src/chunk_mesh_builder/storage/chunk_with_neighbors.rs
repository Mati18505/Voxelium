use cgmath::Vector3;
use shared::entities::{
    BlockID, BlockInChunkPos, BlockStorage, Chunk, CHUNK_SIZE
};

pub struct ChunkWithNeighbors<'a> {
    pub chunk: &'a Chunk,
    pub neighbors: [Option<&'a Chunk>; 6],
}

impl ChunkWithNeighbors<'_> {
    pub fn get_origin_block_storage(&self) -> &BlockStorage {
        self.chunk.get_block_storage()
    }

    /// If pos is in range of origin, get block from this chunk.
    /// Else, if neighbor exist, get block from neighbor.
    /// Else return air.
    /// Does not handle diagonal neighbors.
    pub fn get(&self, pos: Vector3<isize>) -> BlockID {
        let (x, y, z) = (pos.x, pos.y, pos.z);
        let size = CHUNK_SIZE as isize;

        let out_x = x < 0 || x >= size;
        let out_y = y < 0 || y >= size;
        let out_z = z < 0 || z >= size;

        let out_count = out_x as u8 + out_y as u8 + out_z as u8;

        debug_assert!(
            out_count <= 1,
            "Diagonal access not supported: ({}, {}, {})",
            x, y, z
        );

        if (0..size).contains(&x)
        && (0..size).contains(&y)
        && (0..size).contains(&z)
        {
            return self.chunk.get_block_storage().get_block(
                BlockInChunkPos::new(x as usize, y as usize, z as usize),
            );
        }

        let [nz_pos, nz_neg, nx_pos, nx_neg, ny_pos, ny_neg] = self.neighbors;

        if x < 0 {
            if let Some(nx_neg) = nx_neg {
                return nx_neg.get_block_storage().get_block(
                    BlockInChunkPos::new(
                        (size - 1) as usize,
                        y.clamp(0, size - 1) as usize,
                        z.clamp(0, size - 1) as usize,
                    ),
                );
            } else {
                return BlockID::default();
            }
        }

        if x >= size {
            if let Some(nx_pos) = nx_pos {
                return nx_pos.get_block_storage().get_block(
                    BlockInChunkPos::new(
                        0,
                        y.clamp(0, size - 1) as usize,
                        z.clamp(0, size - 1) as usize,
                    ),
                );
            } else {
                return BlockID::default();
            }
        }

        if y < 0 {
            if let Some(ny_neg) = ny_neg {
                return ny_neg.get_block_storage().get_block(
                    BlockInChunkPos::new(
                        x.clamp(0, size - 1) as usize,
                        (size - 1) as usize,
                        z.clamp(0, size - 1) as usize,
                    ),
                );
            } else {
                return BlockID::default();
            }
        }

        if y >= size {
            if let Some(ny_pos) = ny_pos {
                return ny_pos.get_block_storage().get_block(
                    BlockInChunkPos::new(
                        x.clamp(0, size - 1) as usize,
                        0,
                        z.clamp(0, size - 1) as usize,
                    ),
                );
            } else {
                return BlockID::default();
            }
        }

        if z < 0 {
            if let Some(nz_neg) = nz_neg {
                return nz_neg.get_block_storage().get_block(
                    BlockInChunkPos::new(
                        x.clamp(0, size - 1) as usize,
                        y.clamp(0, size - 1) as usize,
                        (size - 1) as usize,
                    ),
                );
            } else {
                return BlockID::default();
            }
        }

        if z >= size {
            if let Some(nz_pos) = nz_pos {
                return nz_pos.get_block_storage().get_block(
                    BlockInChunkPos::new(
                        x.clamp(0, size - 1) as usize,
                        y.clamp(0, size - 1) as usize,
                        0,
                    ),
                );
            } else {
                return BlockID::default();
            }
        }

        unreachable!()
    }
}
