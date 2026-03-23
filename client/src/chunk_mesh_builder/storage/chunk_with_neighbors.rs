use cgmath::Vector3;
use shared::entities::{
    BlockID, BlockInChunkPos, BlockSide, BlockStorage, Chunk, Direction, CHUNK_SIZE,
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
            x,
            y,
            z
        );

        if (0..size).contains(&x) && (0..size).contains(&y) && (0..size).contains(&z) {
            return self
                .chunk
                .get_block_storage()
                .get_block(BlockInChunkPos::new(x as usize, y as usize, z as usize));
        }

        for side in BlockSide::iterator() {
            let dir = Direction::from(*side);
            let dif = dir * CHUNK_SIZE as isize;
            let block_pos: Vector3<isize> = pos - dif;

            if let Some(neighbor) = self.neighbors[*side as usize] {
                if let Some(block) = Self::get_block_checked(neighbor, block_pos) {
                    return block;
                }
            }
        }

        BlockID::default()
    }

    fn get_block_checked(chunk: &Chunk, pos: Vector3<isize>) -> Option<BlockID> {
        let size = CHUNK_SIZE as isize;

        if (0..size).contains(&pos.x) && (0..size).contains(&pos.y) && (0..size).contains(&pos.z) {
            Some(chunk.get_block_storage().get_block(BlockInChunkPos::new(
                pos.x as usize,
                pos.y as usize,
                pos.z as usize,
            )))
        } else {
            None
        }
    }
}
