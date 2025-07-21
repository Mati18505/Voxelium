use bevy::prelude::*;
use shared::entities::ChunkPos;
use shared::entities::ChunkPosGenerator2D;

use crate::bevy_types::AppStates;
use crate::chunk_manager::events::*;
use crate::chunk_manager::resources::*;
use crate::chunk_manager::ChunkState;
use crate::orchestrator::ChunkPosChangedEvent;

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct StreamerConfig {
    /// Horizontal radius (in chunks) within which chunks are loaded.
    pub load_distance: usize,
    /// Horizontal radius (in chunks) within which chunks are rendered.
    pub render_distance: usize,
    /// If true, the engine dynamically loads chunks above and below the player based on vertical position.
    pub dynamic_vertical_loading: bool,
}

impl StreamerConfig {
    pub fn new(load_distance: usize, render_distance: usize) -> Self {
        assert!(render_distance <= load_distance);

        StreamerConfig {
            load_distance,
            render_distance,
            dynamic_vertical_loading: false,
        }
    }
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
            .add_systems(Update, chunk_streamer.run_if(in_state(AppStates::InGame)));
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
    mut world: ResMut<PhysicalWorldResource>,
    config: Res<StreamerConfig>,
) {
    let mut world = &mut world.world;

    for ev in chunk_pos_changed_ev.read() {
        info!("chunk pos: {:?}", ev.chunk_pos);

        let player_pos = ev.chunk_pos;

        // Remove all chunks that are still empty.
        let empty_chunks_in_world: Vec<ChunkPos> = world.get_chunks_with_state(ChunkState::Empty);

        for pos in empty_chunks_in_world {
            chunk_streamer_ev.write(ChunkStreamerRequest::Remove(pos));
        }

        // Load missing chunks within the load distance.
        let to_load = ChunkPosGenerator2D::new(player_pos, config.load_distance);
        for pos in to_load {
            chunk_streamer_ev.write(ChunkStreamerRequest::Load(pos));
        }

        // Update all existing chunks in the world.
        let chunks_in_world: Vec<ChunkPos> = world.chunk_states.keys().copied().collect();

        for pos in chunks_in_world {
            chunk_streamer_ev.write(ChunkStreamerRequest::Update(pos));
        }
    }
}
