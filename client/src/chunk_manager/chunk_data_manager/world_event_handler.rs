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
            .add_event::<ChunkStateTransition>()
            .insert_resource(ChunkStorage::default())
            .add_systems(
                Update,
                (
                    process_world_events.run_if(in_state(AppStates::InGame)),
                    process_state_update_requests.run_if(in_state(AppStates::InGame)),
                    process_transition_events.run_if(in_state(AppStates::InGame)),
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
    pub curr_state: ChunkState,
    pub chunk_status: ChunkDataStatus,
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

    fn create_state_update_request(
        &self,
        pos: ChunkPos,
        storage: &ChunkStorage,
    ) -> StateUpdateRequest {
        let chunk_status = self.create_chunk_status(pos, storage);
        StateUpdateRequest {
            chunk_pos: pos,
            curr_state: storage.get_chunk_state(pos),
            chunk_status,
        }
    }

    fn create_load_chunk_request(
        &self,
        pos: ChunkPos,
        storage: &ChunkStorage,
    ) -> Option<StateUpdateRequest> {
        if storage.get_chunk_state(pos) != ChunkState::Empty {
            return None;
        }

        let mut chunk_status = self.create_chunk_status(pos, storage);
        chunk_status.is_within_load = true;
        Some(StateUpdateRequest {
            chunk_pos: pos,
            curr_state: storage.get_chunk_state(pos),
            chunk_status,
        })
    }

    fn handle_streamer_request(
        &self,
        ev: &ChunkStreamerRequest,
        storage: &mut ChunkStorage,
    ) -> Option<StateUpdateRequest> {
        // TODO: remove only if state is `Empty`?
        match ev {
            ChunkStreamerRequest::Update(pos) => {
                Some(self.create_state_update_request(*pos, storage))
            }
            ChunkStreamerRequest::Remove(pos) => {
                storage.remove_chunk(*pos);
                None
            }
            ChunkStreamerRequest::Load(pos) => self.create_load_chunk_request(*pos, storage),
        }
    }
}

// TODO: get those .clone() out
/// Adds loaded chunks and built meshes to storage.
fn process_world_events(
    mut storage: ResMut<ChunkStorage>,
    mut state_update_req_ev: EventWriter<StateUpdateRequest>,
    mut chunk_loaded_ev: EventReader<ChunkLoaded>,
    mut chunk_streamer_ev: EventReader<ChunkStreamerRequest>,
    config: Res<WorldEventHandlerConfig>,
    chunk_manager_resources: Res<ChunkManagerResource>,
) {
    let mut chunk_update_handler =
        ChunkUpdateHandler::new(chunk_manager_resources.controller_pos, &config);

    for ev in chunk_loaded_ev.read() {
        storage.load(ev.chunk_pos, ev.chunk.clone());

        let req = chunk_update_handler.create_state_update_request(ev.chunk_pos, &storage);
        state_update_req_ev.write(req);
    }

    for ev in chunk_streamer_ev.read() {
        let maybe_req = chunk_update_handler.handle_streamer_request(ev, &mut storage);

        if let Some(req) = maybe_req {
            state_update_req_ev.write(req);
        }
    }
}

fn process_state_update_requests(
    mut state_update_req_ev: EventReader<StateUpdateRequest>,
    mut chunk_state_transition_ev: EventWriter<ChunkStateTransition>,
) {
    for ev in state_update_req_ev.read() {
        let chunk_pos = ev.chunk_pos;
        let prev_state = ev.curr_state;
        let next_state = prev_state.get_next_chunk_state(ev.chunk_status);
        let transition = prev_state.get_chunk_transition(next_state);

        if let Some(transition) = transition {
            trace!("Chunk transition in chunk {chunk_pos:?}: {prev_state:?} -> {next_state:?}");

            chunk_state_transition_ev.write(ChunkStateTransition {
                chunk_pos,
                transition,
                new_state: next_state,
            });
        }
    }
}

fn process_transition_events(
    mut storage: ResMut<ChunkStorage>,
    mut chunk_load_req_ev: EventWriter<ChunkLoaderRequest>,
    mut chunk_state_transition_ev: EventReader<ChunkStateTransition>,
    chunk_manager_resources: Res<ChunkManagerResource>,
) {
    log::info!("{:?}", chunk_manager_resources.controller_pos);

    for ev in chunk_state_transition_ev.read() {
        let pos = ev.chunk_pos;

        use ChunkState::*;
        let t: ChunkTransition = ev.transition;

        if ev.transition.to != Loaded {
            dbg!(ev.chunk_pos, ev.transition);
            storage.change_state(pos, ev.transition);
        }

        match (t.from, t.to) {
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
    }
}
