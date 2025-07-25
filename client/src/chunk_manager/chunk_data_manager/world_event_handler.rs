use std::collections::HashSet;

use bevy::{log, prelude::*};
use shared::entities::*;

use crate::bevy_types::AppStates;
use crate::chunk_manager::ChunkManagerConfig;

use super::chunk_state::*;
use super::chunk_storage::ChunkStorage;
use super::chunk_streamer::StreamerConfig;
use super::events::*;
use super::resources::*;

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct WorldEventHandlerConfig {
    /// Horizontal radius (in chunks) within which chunks are loaded.
    pub load_distance: usize,
    /// If true, the engine dynamically loads chunks above and below the player based on vertical position.
    pub dynamic_vertical_loading: bool,
}

pub struct WorldEventHandlerPlugin {
    config: WorldEventHandlerConfig,
}
impl WorldEventHandlerPlugin {
    pub fn new(config: WorldEventHandlerConfig) -> Self {
        Self { config }
    }
}

impl Plugin for WorldEventHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .add_event::<StateUpdateRequest>()
            .insert_resource(ChunkStorage::default())
            .add_systems(
                Update,
                (
                    process_world_events.run_if(in_state(AppStates::InGame)),
                    process_state_update_requests.run_if(in_state(AppStates::InGame)),
                ),
            );
    }
}

struct ChunkUpdateHandler<'a> {
    controller_pos: ChunkPos,
    config: &'a WorldEventHandlerConfig,
}

/// `world_event_handler` request to update the chunk state based on its status.
#[derive(Event, Debug)]
struct StateUpdateRequest {
    pub chunk_pos: ChunkPos,
}

impl<'a> ChunkUpdateHandler<'a> {
    fn new(controller_pos: ChunkPos, config: &'a WorldEventHandlerConfig) -> Self {
        Self {
            controller_pos,
            config,
        }
    }

    fn is_player_within_distance(&self, pos: ChunkPos, distance_in_chunks: usize) -> bool {
        match self.config.dynamic_vertical_loading {
            true => pos.is_within_distance(self.controller_pos, distance_in_chunks),
            false => pos.is_within_distance_2d(self.controller_pos, distance_in_chunks),
        }
    }

    fn create_chunk_status(&self, pos: ChunkPos, storage: &ChunkStorage) -> ChunkDataStatus {
        let is_within_load = self.is_player_within_distance(pos, self.config.load_distance);
        let loaded = storage.is_loaded(pos);

        ChunkDataStatus {
            is_within_load,
            loaded,
        }
    }
}

// TODO: get those .clone() out
/// Adds loaded chunks and built meshes to storage.
fn process_world_events(
    mut state_update_req_ev: EventWriter<StateUpdateRequest>,
    mut chunk_loaded_ev: EventReader<ChunkLoaded>,
    mut chunk_streamer_ev: EventReader<ChunkStreamerRequest>,
) {
    let mut chunks_to_update = HashSet::<ChunkPos>::default();
    chunks_to_update.extend(chunk_loaded_ev.read().map(|loaded| loaded.chunk_pos));
    chunks_to_update.extend(chunk_streamer_ev.read().map(|rq| rq.chunk_pos));

    for chunk_pos in chunks_to_update {
        let req = StateUpdateRequest { chunk_pos };

        state_update_req_ev.write(req);
    }
}

fn process_state_update_requests(
    mut state_update_req_ev: EventReader<StateUpdateRequest>,
    mut storage: ResMut<ChunkStorage>,
    mut chunk_load_req_ev: EventWriter<ChunkLoaderRequest>,
    config: Res<WorldEventHandlerConfig>,
    chunk_manager_resources: Res<ChunkManagerResource>,
) {
    let mut chunk_update_handler =
        ChunkUpdateHandler::new(chunk_manager_resources.controller_pos, &config);

    for ev in state_update_req_ev.read() {
        let chunk_pos = ev.chunk_pos;
        let chunk_status = chunk_update_handler.create_chunk_status(chunk_pos, &storage);
        let prev_state = storage.get_chunk_state(chunk_pos);
        let next_state = prev_state.get_next_chunk_state(chunk_status);
        let transition = prev_state.get_chunk_transition(next_state);

        if let Some(transition) = transition {
            trace!("Chunk transition in chunk {chunk_pos:?}: {prev_state:?} -> {next_state:?}");

            process_transition(&mut storage, &mut chunk_load_req_ev, chunk_pos, transition);
        }
    }
}

fn process_transition(
    mut storage: &mut ResMut<ChunkStorage>,
    mut chunk_load_req_ev: &mut EventWriter<ChunkLoaderRequest>,
    pos: ChunkPos,
    transition: ChunkTransition,
) {
    use ChunkState::*;

    if transition.to != Loaded {
        storage.change_state(pos, transition);
    } else {
        storage.load(pos, Chunk::default());
    }

    match (transition.from, transition.to) {
        (Empty, Loading) => {
            log::debug!("Started loading chunk {:?}", pos);
            chunk_load_req_ev.write(ChunkLoaderRequest::Load(pos));
        }
        (Loading, Empty) => {
            log::debug!("Canceled loading chunk {:?}", pos);
            chunk_load_req_ev.write(ChunkLoaderRequest::CancelLoading(pos));
        }
        (_, Loaded) => {
            log::debug!("Loaded chunk {:?}", pos);

            // TODO: send event to ChunkMesh manager.
        }
        (_, Empty) => {
            log::debug!("Removed chunk {:?}", pos);
            // TODO: send event to ChunkMesh manager.
        }
        _ => unreachable!(),
    }

    // dbg!(storage);
}
