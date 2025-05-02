pub type BlockID = u8;
pub const CHUNK_SIZE: usize = 16;

use cgmath::Vector3;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ChunkPos(pub Vector3<isize>);

#[derive(Debug, Clone, Copy)] 
pub struct BlockPos(pub Vector3<isize>);

#[derive(Debug, Clone, Copy)] 
pub struct BlockInChunkPos(pub Vector3<usize>);

impl Deref for ChunkPos {
    type Target = Vector3<isize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BlockPos {
    type Target = Vector3<isize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BlockInChunkPos {
    type Target = Vector3<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl From<BlockPos> for ChunkPos {
    fn from(block_pos: BlockPos) -> Self {
        let pos = Vector3 {
            x: (block_pos.x as f32 / CHUNK_SIZE as f32).floor() as isize * CHUNK_SIZE as isize,
            y: (block_pos.y as f32 / CHUNK_SIZE as f32).floor() as isize * CHUNK_SIZE as isize,
            z: 0,
        };
        ChunkPos {
            0: pos
        }
    }
}

macro_rules! same_sign {
    ($first:expr, $sec:expr) => {
        ($first < 0 && $sec < 0)
        || ($first >= 0 && $sec >= 0)
    }
}

impl BlockInChunkPos {
    pub fn new(world_pos: BlockPos, chunk_pos: ChunkPos) -> Self {
        // Block and chunk have the same sign.
        assert!(same_sign!(world_pos.x, chunk_pos.x));
        assert!(same_sign!(world_pos.y, chunk_pos.y));
        assert!(same_sign!(world_pos.z, chunk_pos.z));

        let in_chunk_pos = world_pos.0 - chunk_pos.0;
        let in_chunk_pos: Vector3<usize> = in_chunk_pos.cast().unwrap();
        
        // Block must be in chunk.
        assert!(in_chunk_pos.x < CHUNK_SIZE);
        assert!(in_chunk_pos.y < CHUNK_SIZE);
        assert!(in_chunk_pos.z < CHUNK_SIZE);

        BlockInChunkPos(in_chunk_pos)
    }

    pub fn index(&self) -> usize {
        self.y * CHUNK_SIZE * CHUNK_SIZE + self.z * CHUNK_SIZE + self.x
    }
}