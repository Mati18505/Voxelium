use bevy::prelude::*;

use crate::chunk_manager::physical_world::PhysicalWorld;
use shared::entities::ChunkPos;

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

#[derive(Resource, Default)]
pub struct PhysicalWorldResource {
    pub world: PhysicalWorld,
}
