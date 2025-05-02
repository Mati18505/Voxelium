pub type BlockID = u8;
pub const CHUNK_SIZE: usize = 16;

use cgmath::Vector3;
use std::ops::Deref;

pub type BlockPos = Vector3<isize>;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ChunkPos(Vector3<isize>);

#[derive(Debug, Clone, Copy, PartialEq)] 
pub struct BlockInChunkPos(Vector3<usize>);

impl ChunkPos {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        assert!(x % CHUNK_SIZE as isize == 0, "ChunkPos must be multiple of CHUNK_SIZE. x = {}", x);
        assert!(y % CHUNK_SIZE as isize == 0, "ChunkPos must be multiple of CHUNK_SIZE. y = {}", y);
        assert!(z % CHUNK_SIZE as isize == 0, "ChunkPos must be multiple of CHUNK_SIZE. z = {}", z);

        ChunkPos(Vector3::new(x, y, z))
    }
}

impl Deref for ChunkPos {
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

impl BlockInChunkPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        assert!(x < CHUNK_SIZE, "Block must be in chunk. x = {}", x);
        assert!(y < CHUNK_SIZE, "Block must be in chunk. y = {}", y);
        assert!(z < CHUNK_SIZE, "Block must be in chunk. z = {}", z);
        
        BlockInChunkPos(Vector3::new(x, y, z))
    }

    pub fn index(&self) -> usize {
        self.y * CHUNK_SIZE * CHUNK_SIZE + self.z * CHUNK_SIZE + self.x
    }

    pub fn from_index(index: usize) -> Self {
        let y = index / (CHUNK_SIZE * CHUNK_SIZE);
        let rem = index % (CHUNK_SIZE * CHUNK_SIZE);
        let z = rem / CHUNK_SIZE;
        let x = rem % CHUNK_SIZE;

        BlockInChunkPos::new(x, y, z)
    }
}

impl From<BlockPos> for BlockInChunkPos {
    fn from(world_pos: BlockPos) -> Self {
        let chunk_pos = ChunkPos::from(world_pos);

        let in_chunk_pos = world_pos - chunk_pos.0;
        let in_chunk_pos: Vector3<usize> = in_chunk_pos.cast().expect("Block is outside this chunk.");
        
        Self::new(in_chunk_pos.x, in_chunk_pos.y, in_chunk_pos.z)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BlockSide { FRONT, BACK, LEFT, RIGHT, TOP, BOTTOM }

// Z=UP, right handed
impl From<BlockSide> for Vector3<isize> {
    fn from(side: BlockSide) -> Self {
        match side {
            BlockSide::FRONT => Vector3::new(0, 1, 0),
            BlockSide::BACK => Vector3::new(0,  -1, 0),
            BlockSide::RIGHT => Vector3::new(1, 0, 0),
            BlockSide::LEFT => Vector3::new(-1, 0, 0),
            BlockSide::TOP => Vector3::new(0, 0, 1),
            BlockSide::BOTTOM => Vector3::new(0, 0, -1),
        }
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
    fn test_block_in_chunk_pos_from_index() {
        let idx = 0;
        let expected = BlockInChunkPos::new(0, 0, 0);
        assert_eq!(BlockInChunkPos::from_index(idx), expected);

        let idx = 15;
        let expected = BlockInChunkPos::new(15, 0, 0);
        assert_eq!(BlockInChunkPos::from_index(idx), expected);

        let idx = 15 * CHUNK_SIZE;
        let expected = BlockInChunkPos::new(0, 0, 15);
        assert_eq!(BlockInChunkPos::from_index(idx), expected);

        let idx = 15 * CHUNK_SIZE * CHUNK_SIZE;
        let expected = BlockInChunkPos::new(0, 15, 0);
        assert_eq!(BlockInChunkPos::from_index(idx), expected);
    }

    #[test]
    fn test_block_in_chunk_pos_from_world_pos() {
        let world_pos = BlockPos::new(5, 0, 0);
        let block_in_chunk = BlockInChunkPos::from(world_pos);
        assert_eq!(block_in_chunk, BlockInChunkPos::new(5, 0, 0));

        let world_pos = BlockPos::new(20, 16, 16);
        let block_in_chunk = BlockInChunkPos::from(world_pos);
        assert_eq!(block_in_chunk, BlockInChunkPos::new(4, 0, 0));

        let world_pos = BlockPos::new(-1, -1, -1);
        let block_in_chunk = BlockInChunkPos::from(world_pos);
        assert_eq!(block_in_chunk, BlockInChunkPos::new(15, 15, 15));

        let world_pos = BlockPos::new(-5, -5, -5);
        let block_in_chunk = BlockInChunkPos::from(world_pos);
        assert_eq!(block_in_chunk, BlockInChunkPos::new(11, 11, 11));

        let world_pos = BlockPos::new(-17, -5, -5);
        let block_in_chunk = BlockInChunkPos::from(world_pos);
        assert_eq!(block_in_chunk, BlockInChunkPos::new(15, 11, 11));
    }
}