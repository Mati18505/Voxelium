pub type BlockID = u8;
pub const CHUNK_SIZE: usize = 16;

use cgmath::Vector3;
use std::{
    ops::{Deref, DerefMut},
    slice::Iter,
};

pub type BlockPos = Vector3<isize>;
pub type Direction = Vector3<isize>;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ChunkPos(Vector3<isize>);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlockInChunkPos(Vector3<usize>);

impl ChunkPos {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        assert!(
            x % CHUNK_SIZE as isize == 0,
            "ChunkPos must be multiple of CHUNK_SIZE. x = {x}",
        );
        assert!(
            y % CHUNK_SIZE as isize == 0,
            "ChunkPos must be multiple of CHUNK_SIZE. y = {y}",
        );
        assert!(
            z % CHUNK_SIZE as isize == 0,
            "ChunkPos must be multiple of CHUNK_SIZE. z = {z}",
        );

        ChunkPos(Vector3::new(x, y, z))
    }

    pub fn is_within_distance(&self, other: ChunkPos, mut dist_in_chunks: usize) -> bool {
        dist_in_chunks *= CHUNK_SIZE;

        let z_start = other.z - dist_in_chunks as isize;
        let z_end = other.z + dist_in_chunks as isize;
        let y_start = other.y - dist_in_chunks as isize;
        let y_end = other.y + dist_in_chunks as isize;
        let x_start = other.x - dist_in_chunks as isize;
        let x_end = other.x + dist_in_chunks as isize;

        if self.x >= x_start
            && self.x <= x_end
            && self.y >= y_start
            && self.y <= y_end
            && self.z >= z_start
            && self.z <= z_end
        {
            return true;
        }

        false
    }

    pub fn is_within_distance_2d(&self, other: ChunkPos, mut dist_in_chunks: usize) -> bool {
        dist_in_chunks *= CHUNK_SIZE;

        let z_start = other.z - dist_in_chunks as isize;
        let z_end = other.z + dist_in_chunks as isize;
        let x_start = other.x - dist_in_chunks as isize;
        let x_end = other.x + dist_in_chunks as isize;

        if self.x >= x_start && self.x <= x_end && self.z >= z_start && self.z <= z_end {
            return true;
        }

        false
    }
}

impl Deref for ChunkPos {
    type Target = Vector3<isize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChunkPos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for BlockInChunkPos {
    type Target = Vector3<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BlockInChunkPos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
        assert!(x < CHUNK_SIZE, "Block must be in chunk. x = {x}");
        assert!(y < CHUNK_SIZE, "Block must be in chunk. y = {y}");
        assert!(z < CHUNK_SIZE, "Block must be in chunk. z = {z}");

        BlockInChunkPos(Vector3::new(x, y, z))
    }

    pub fn index(&self) -> usize {
        self.z * CHUNK_SIZE * CHUNK_SIZE + self.y * CHUNK_SIZE + self.x
    }

    pub fn from_index(index: usize) -> Self {
        let z = index / (CHUNK_SIZE * CHUNK_SIZE);
        let rem = index % (CHUNK_SIZE * CHUNK_SIZE);
        let y = rem / CHUNK_SIZE;
        let x = rem % CHUNK_SIZE;

        BlockInChunkPos::new(x, y, z)
    }

    pub fn checked_add(&self, dir: Vector3<isize>) -> Option<Self> {
        let x = self.x.checked_add_signed(dir.x);
        let y = self.y.checked_add_signed(dir.y);
        let z = self.z.checked_add_signed(dir.z);

        if let (Some(x), Some(y), Some(z)) = (x, y, z) {
            if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
                return Some(BlockInChunkPos::new(x, y, z));
            }
        }

        None
    }
}

impl From<BlockPos> for BlockInChunkPos {
    fn from(world_pos: BlockPos) -> Self {
        let chunk_pos = ChunkPos::from(world_pos);

        let in_chunk_pos = world_pos - chunk_pos.0;
        let in_chunk_pos: Vector3<usize> =
            in_chunk_pos.cast().expect("Block is outside this chunk.");

        Self::new(in_chunk_pos.x, in_chunk_pos.y, in_chunk_pos.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockSide {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

impl BlockSide {
    pub fn iterator() -> Iter<'static, BlockSide> {
        use BlockSide::*;

        static SIDES: [BlockSide; 6] = [Front, Back, Left, Right, Top, Bottom];
        SIDES.iter()
    }
}

// Y=up, right handed (like Bevy)
impl From<BlockSide> for Direction {
    fn from(side: BlockSide) -> Self {
        match side {
            BlockSide::Front => Direction::new(0, 0, 1),
            BlockSide::Back => Direction::new(0, 0, -1),
            BlockSide::Right => Direction::new(1, 0, 0),
            BlockSide::Left => Direction::new(-1, 0, 0),
            BlockSide::Top => Direction::new(0, 1, 0),
            BlockSide::Bottom => Direction::new(0, -1, 0),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chunk_pos_from_block_pos() {
        let block_pos = BlockPos::new(0, 0, 0);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(0, 0, 0));

        let block_pos = BlockPos::new(15, 15, 15);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(0, 0, 0));

        let block_pos = BlockPos::new(16, 16, 16);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(16, 16, 16));
    }

    #[test]
    fn test_chunk_pos_from_block_pos_negative() {
        let block_pos = BlockPos::new(-1, -1, -1);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(-16, -16, -16));

        let block_pos = BlockPos::new(-16, -16, -16);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(-16, -16, -16));

        let block_pos = BlockPos::new(-17, -17, -17);
        assert_eq!(ChunkPos::from(block_pos), ChunkPos::new(-32, -32, -32));
    }

    #[test]
    fn test_block_in_chunk_pos_index() {
        let pos = BlockInChunkPos::new(0, 0, 0);
        assert_eq!(pos.index(), 0);

        let pos = BlockInChunkPos::new(15, 0, 0);
        assert_eq!(pos.index(), 15);

        // Y=up
        let pos = BlockInChunkPos::new(0, 15, 0);
        assert_eq!(pos.index(), 15 * CHUNK_SIZE);

        let pos = BlockInChunkPos::new(0, 0, 15);
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

        // Y=up
        let idx = 15 * CHUNK_SIZE;
        let expected = BlockInChunkPos::new(0, 15, 0);
        assert_eq!(BlockInChunkPos::from_index(idx), expected);

        let idx = 15 * CHUNK_SIZE * CHUNK_SIZE;
        let expected = BlockInChunkPos::new(0, 0, 15);
        assert_eq!(BlockInChunkPos::from_index(idx), expected);
    }

    #[test]
    fn test_index_from_index() {
        let idx = 0;
        assert_eq!(BlockInChunkPos::from_index(idx).index(), idx);

        let idx = 15;
        assert_eq!(BlockInChunkPos::from_index(idx).index(), idx);

        let idx = 15 * CHUNK_SIZE;
        assert_eq!(BlockInChunkPos::from_index(idx).index(), idx);

        let idx = 15 * CHUNK_SIZE * CHUNK_SIZE;
        assert_eq!(BlockInChunkPos::from_index(idx).index(), idx);
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

    #[test]
    fn test_checked_add() {
        let pos = BlockInChunkPos::new(0, 0, 0);
        let add = Vector3::new(-1, -1, -1);
        assert_eq!(pos.checked_add(add), None);

        let pos = BlockInChunkPos::new(15, 15, 15);
        let add = Vector3::new(-1, -1, -1);
        assert_eq!(pos.checked_add(add), Some(BlockInChunkPos::new(14, 14, 14)));

        let pos = BlockInChunkPos::new(15, 15, 15);
        let add = Vector3::new(1, 1, 1);
        assert_eq!(pos.checked_add(add), None);
    }

    #[test]
    fn test_is_within_distance() {
        let pos1 = ChunkPos::new(0, 0, 0);
        let pos2 = ChunkPos::new(CHUNK_SIZE as isize * 8, 0, 0);

        assert!(!pos1.is_within_distance(pos2, 1));
        assert!(pos1.is_within_distance(pos2, 8));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}