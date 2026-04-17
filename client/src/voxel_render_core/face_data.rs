use shared::entities::{BlockInChunkPos, BlockSide};

pub type StorageIndex = u32;

#[derive(Debug, Clone)]
pub struct FaceData {
    pub facing_side: BlockSide,
    pub block_pos: BlockInChunkPos,
    pub uv_2: StorageIndex,
}
