pub type BlockID = u8;
pub const CHUNK_SIZE: usize = 16;

use cgmath::Vector3;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ChunkPos(Vector3<isize>);

#[derive(Debug, Clone, Copy, PartialEq)] 
pub struct BlockPos(Vector3<isize>);

#[derive(Debug, Clone, Copy, PartialEq)] 
pub struct BlockInChunkPos(Vector3<usize>);

impl ChunkPos {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        // ChunkPos must be multiple of CHUNK_SIZE.
        assert!(x % CHUNK_SIZE as isize == 0);
        assert!(y % CHUNK_SIZE as isize == 0);
        assert!(z % CHUNK_SIZE as isize == 0);

        ChunkPos(Vector3::new(x, y, z))
    }
}

impl BlockPos {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        BlockPos(Vector3::new(x, y, z))
    }
}

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
        let x = (block_pos.x as f32 / CHUNK_SIZE as f32).floor() as isize * CHUNK_SIZE as isize;
        let y = (block_pos.y as f32 / CHUNK_SIZE as f32).floor() as isize * CHUNK_SIZE as isize;
        let z = (block_pos.z as f32 / CHUNK_SIZE as f32).floor() as isize * CHUNK_SIZE as isize;
        
        ChunkPos::new(x, y, z)
    }
}

macro_rules! same_sign {
    ($first:expr, $sec:expr) => {
        ($first < 0 && $sec < 0)
        || ($first >= 0 && $sec >= 0)
    }
}

impl BlockInChunkPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        // Block must be in chunk.
        assert!(x < CHUNK_SIZE);
        assert!(y < CHUNK_SIZE);
        assert!(z < CHUNK_SIZE);
        
        BlockInChunkPos(Vector3::new(x, y, z))
    }

    pub fn from_world_and_chunk(world_pos: BlockPos, chunk_pos: ChunkPos) -> Self {
        // Block and chunk have the same sign.
        assert!(same_sign!(world_pos.x, chunk_pos.x));
        assert!(same_sign!(world_pos.y, chunk_pos.y));
        assert!(same_sign!(world_pos.z, chunk_pos.z));

        let in_chunk_pos = world_pos.0 - chunk_pos.0;
        let in_chunk_pos: Vector3<usize> = in_chunk_pos.cast().expect("Block is outside this chunk.");
        
        Self::new(in_chunk_pos.x, in_chunk_pos.y, in_chunk_pos.z)
    }

    pub fn index(&self) -> usize {
        self.y * CHUNK_SIZE * CHUNK_SIZE + self.z * CHUNK_SIZE + self.x
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chunk_pos_from_block_pos() {
        let block_pos = BlockPos::new(0, 0, 0);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(0, 0 ,0));

        let block_pos = BlockPos::new(15, 15, 15);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(0, 0 ,0));

        let block_pos = BlockPos::new(16, 16, 16);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(16, 16 ,16));
    }

    #[test]
    fn test_chunk_pos_from_block_pos_negative() { 
        let block_pos = BlockPos::new(-1, -1, -1);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(-16, -16 ,-16));

        let block_pos = BlockPos::new(-16, -16, -16);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(-16, -16 ,-16));

        let block_pos = BlockPos::new(-17, -17, -17);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(-32, -32 ,-32));
    }

    #[test]
    fn test_block_in_chunk_pos_index() {
        let pos = BlockInChunkPos::new(0, 0, 0);
        assert_eq!(pos.index(), 0);

        let pos = BlockInChunkPos::new(15, 0, 0);
        assert_eq!(pos.index(), 15);

        let pos = BlockInChunkPos::new(0, 0, 15);
        assert_eq!(pos.index(), 15 * CHUNK_SIZE);

        let pos = BlockInChunkPos::new(0, 15, 0);
        assert_eq!(pos.index(), 15 * CHUNK_SIZE * CHUNK_SIZE);
    }

    #[test]
    fn test_block_in_chunk_pos_from_world_pos() {
        let chunk_pos = ChunkPos::new(0, 0, 0);
        let world_pos = BlockPos::new(5, 0, 0);
        let block_in_chunk = BlockInChunkPos::from_world_and_chunk(world_pos, chunk_pos);
        assert_eq!(block_in_chunk, BlockInChunkPos::new(5, 0, 0));

        let chunk_pos = ChunkPos::new(16, 16, 16);
        let world_pos = BlockPos::new(20, 16, 16);
        let block_in_chunk = BlockInChunkPos::from_world_and_chunk(world_pos, chunk_pos);
        assert_eq!(block_in_chunk, BlockInChunkPos::new(4, 0, 0));

        let chunk_pos = ChunkPos::new(-16, -16, -16);
        let world_pos = BlockPos::new(-1, -1, -1);
        let block_in_chunk = BlockInChunkPos::from_world_and_chunk(world_pos, chunk_pos);
        assert_eq!(block_in_chunk, BlockInChunkPos::new(15, 15, 15));

        let chunk_pos = ChunkPos::new(-16, -16, -16);
        let world_pos = BlockPos::new(-5, -5, -5);
        let block_in_chunk = BlockInChunkPos::from_world_and_chunk(world_pos, chunk_pos);
        assert_eq!(block_in_chunk, BlockInChunkPos::new(11, 11, 11));

        let chunk_pos = ChunkPos::new(-32, -16, -16);
        let world_pos = BlockPos::new(-17, -5, -5);
        let block_in_chunk = BlockInChunkPos::from_world_and_chunk(world_pos, chunk_pos);
        assert_eq!(block_in_chunk, BlockInChunkPos::new(15, 11, 11));
    }
}