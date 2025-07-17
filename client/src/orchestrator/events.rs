use bevy::ecs::event::Event;
use shared::entities::{BlockPos, BlockType};

// Controller update OR world chunk update.
#[derive(Event, Debug)]
pub struct LookedAtBlockChangedEvent {
    pub block_pos: BlockPos,
    pub block_type: BlockType,
}
