pub type BlockID = u8;
pub const CHUNK_SIZE: usize = 16;

use cgmath::Vector3;
use std::ops::Deref;

#[derive(Hash, Eq, PartialEq)]
pub struct ChunkPos(pub Vector3<isize>);
pub struct BlockPos(pub Vector3<isize>);

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

impl From<&BlockPos> for ChunkPos {
    fn from(block_pos: &BlockPos) -> Self {
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