use bevy::ecs::event::Event;
use shared::entities::{BlockPos, BlockType, ChunkPos};

/// Controller update OR world chunk update.
#[derive(Event, Debug)]
pub struct LookedAtBlockChangedEvent {
    pub block_pos: BlockPos,
    pub block_type: BlockType,
}

/// Controller position in chunks changed.
#[derive(Event, Debug)]
pub struct ChunkPosChangedEvent {
    pub chunk_pos: ChunkPos,
}
