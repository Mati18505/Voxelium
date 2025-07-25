use bevy::prelude::*;

use shared::entities::*;

#[derive(Resource)]
pub struct ChunkManagerResource {
    /// Stores current player position.
    pub controller_pos: ChunkPos,
}

impl Default for ChunkManagerResource {
    fn default() -> Self {
        Self {
            controller_pos: ChunkPos::new(0, 0, 0),
        }
    }
}
