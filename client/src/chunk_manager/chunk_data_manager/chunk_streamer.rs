use bevy::log;
use bevy::prelude::*;
use shared::entities::*;

use super::chunk_storage::ChunkStorage;
use super::events::*;
use super::resources::*;

use super::chunk_state::*;
use crate::bevy_types::AppStates;
use crate::orchestrator::ChunkPosChangedEvent;

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct StreamerConfig {
    /// Horizontal radius (in chunks) within which chunks are loaded.
    pub load_distance: usize,
    /// If true, the engine dynamically loads chunks above and below the player based on vertical position.
    pub dynamic_vertical_loading: bool,
}

pub struct ChunkStreamerPlugin {
    config: StreamerConfig,
}

impl ChunkStreamerPlugin {
    pub fn new(config: StreamerConfig) -> Self {
        Self { config }
    }
}

impl Plugin for ChunkStreamerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .insert_resource(ChunkManagerResource::default())
            .add_event::<ChunkStreamerRequest>()
            .add_systems(
                Update,
                (chunk_streamer, update_controller_pos).run_if(in_state(AppStates::InGame)),
            );
    }
}

fn update_controller_pos(
    mut chunk_pos_changed_ev: EventReader<ChunkPosChangedEvent>,
    mut chunk_manager_resource: ResMut<ChunkManagerResource>,
) {
    if let Some(ev) = chunk_pos_changed_ev.read().last() {
        if chunk_manager_resource.controller_pos != ev.chunk_pos {
            chunk_manager_resource.controller_pos = ev.chunk_pos;
        }
    }
}

// TODO: use Changed<> ?
/// Manages loaded chunks in world based on player position.
fn chunk_streamer(
    mut chunk_pos_changed_ev: EventReader<ChunkPosChangedEvent>,
    mut chunk_streamer_ev: EventWriter<ChunkStreamerRequest>,
    config: Res<StreamerConfig>,
) {
    for ev in chunk_pos_changed_ev.read() {
        info!("chunk pos: {:?}", ev.chunk_pos);

        let player_pos = ev.chunk_pos;

        // Update all chunks within the load distance.
        let to_load = ChunkPosGenerator2D::new(player_pos, config.load_distance);
        for chunk_pos in to_load {
            chunk_streamer_ev.write(ChunkStreamerRequest { chunk_pos });
        }
    }
}
