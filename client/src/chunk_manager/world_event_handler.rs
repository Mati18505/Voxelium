use bevy::{log, prelude::*};
use shared::entities::{Chunk, ChunkPos, ChunkRepository};

use crate::bevy_types::AppStates;
use crate::chunk_manager::ChunkTransition;
use crate::chunk_manager::{
    chunk_streamer::StreamerConfig, events::*, physical_world::PhysicalWorld, resources::*,
    ChunkState, ChunkStatus,
};

pub struct WorldEventHandlerPlugin;
impl Plugin for WorldEventHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StateUpdateRequest>()
            .insert_resource(PhysicalWorldResource::default())
            .add_systems(
                Update,
                (
                    process_world_events.run_if(in_state(AppStates::InGame)),
                    process_transition_events.run_if(in_state(AppStates::InGame)),
                ),
            );
    }
}

struct ChunkUpdateHandler<'a> {
    controller_pos: ChunkPos,
    config: &'a StreamerConfig,
}

impl<'a> ChunkUpdateHandler<'a> {
    fn new(controller_pos: ChunkPos, config: &'a StreamerConfig) -> Self {
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

    fn create_chunk_status(&self, pos: ChunkPos, world: &PhysicalWorld) -> ChunkStatus {
        let is_within_render = self.is_player_within_distance(pos, self.config.render_distance);
        let is_within_load = self.is_player_within_distance(pos, self.config.load_distance);
        let loaded = world.get_chunk(pos).is_some();
        let mesh_built = world.get_chunk_mesh(pos).is_some();
        let needs_rebuild = world.get_chunk_need_rebuild(pos);

        ChunkStatus {
            is_within_render,
            is_within_load,
            loaded,
            mesh_built,
            needs_rebuild,
        }
    }

    fn create_state_update_request(
        &self,
        pos: ChunkPos,
        world: &PhysicalWorld,
    ) -> StateUpdateRequest {
        let chunk_status = self.create_chunk_status(pos, world);
        StateUpdateRequest {
            chunk_pos: pos,
            curr_state: world.get_chunk_state(pos),
            chunk_status,
        }
    }

    fn create_load_chunk_request(
        &self,
        pos: ChunkPos,
        world: &PhysicalWorld,
    ) -> Option<StateUpdateRequest> {
        if world.get_chunk_state(pos) != ChunkState::Empty {
            return None;
        }

        let mut chunk_status = self.create_chunk_status(pos, world);
        chunk_status.is_within_load = true;
        Some(StateUpdateRequest {
            chunk_pos: pos,
            curr_state: world.get_chunk_state(pos),
            chunk_status,
        })
    }

    fn apply_chunk_loaded(&self, ev: &ChunkLoaded, world: &mut PhysicalWorld) {
        if world.get_chunk_state(ev.chunk_pos) == ChunkState::Loading {
            world.set_chunk(ev.chunk_pos, ev.chunk.clone());
        }
    }

    fn apply_chunk_built(&self, ev: &ChunkBuilt, world: &mut PhysicalWorld) {
        if world.get_chunk_state(ev.chunk_pos) == ChunkState::ToDraw {
            world.add_chunk_mesh(ev.chunk_pos, ev.chunk_mesh.clone());
        }
    }

    fn handle_streamer_request(
        &self,
        ev: &ChunkStreamerRequest,
        world: &mut PhysicalWorld,
    ) -> Option<StateUpdateRequest> {
        // TODO: remove only if state is `Empty`?
        match ev {
            ChunkStreamerRequest::Update(pos) => {
                Some(self.create_state_update_request(*pos, world))
            }
            ChunkStreamerRequest::Remove(pos) => {
                world.remove_chunk(*pos);
                None
            }
            ChunkStreamerRequest::Load(pos) => self.create_load_chunk_request(*pos, world),
        }
    }

    fn get_chunk_to_draw(&self, pos: ChunkPos, world: &mut PhysicalWorld) -> Chunk {
        world.remove_chunk_need_rebuild(pos);

        world
            .get_chunk(pos)
            .expect("Chunk is passed to builder, but it is not loaded.")
            .clone()
    }
}

// TODO: get those .clone() out
/// Adds loaded chunks and built meshes to world.
fn process_world_events(
    mut world: ResMut<PhysicalWorldResource>,
    mut state_update_req_ev: EventWriter<StateUpdateRequest>,
    mut chunk_loaded_ev: EventReader<ChunkLoaded>,
    mut chunk_built_ev: EventReader<ChunkBuilt>,
    mut chunk_streamer_ev: EventReader<ChunkStreamerRequest>,
    config: Res<StreamerConfig>,
    chunk_manager_resources: Res<ChunkManagerResource>,
) {
    let mut chunk_update_handler =
        ChunkUpdateHandler::new(chunk_manager_resources.controller_pos, &config);

    for ev in chunk_loaded_ev.read() {
        chunk_update_handler.apply_chunk_loaded(ev, &mut world.world);

        let req = chunk_update_handler.create_state_update_request(ev.chunk_pos, &world.world);
        state_update_req_ev.write(req);
    }

    for ev in chunk_built_ev.read() {
        chunk_update_handler.apply_chunk_built(ev, &mut world.world);

        let req = chunk_update_handler.create_state_update_request(ev.chunk_pos, &world.world);
        state_update_req_ev.write(req);
    }

    for ev in chunk_streamer_ev.read() {
        let maybe_req = chunk_update_handler.handle_streamer_request(ev, &mut world.world);

        if let Some(req) = maybe_req {
            state_update_req_ev.write(req);
        }
    }
}

fn process_transition_events(
    mut world: ResMut<PhysicalWorldResource>,
    mut chunk_load_req_ev: EventWriter<ChunkLoaderRequest>,
    mut chunk_build_req_ev: EventWriter<ChunkBuilderRequest>,
    mut chunk_entity_req_ev: EventWriter<ChunkEntityEvent>,
    mut state_update_req_ev: EventWriter<StateUpdateRequest>,
    mut chunk_state_transition_ev: EventReader<ChunkStateTransition>,
    config: Res<StreamerConfig>,
    chunk_manager_resources: Res<ChunkManagerResource>,
) {
    let mut chunk_update_handler =
        ChunkUpdateHandler::new(chunk_manager_resources.controller_pos, &config);

    for ev in chunk_state_transition_ev.read() {
        let pos = ev.chunk_pos;
        let mut world = &mut world.world;
        world.set_chunk_state(pos, ev.new_state);

        use ChunkTransition::*;

        match ev.transition {
            EmptyToLoading => {
                chunk_load_req_ev.write(ChunkLoaderRequest::Load(pos));
            }
            LoadingToEmpty => {
                chunk_load_req_ev.write(ChunkLoaderRequest::CancelLoading(pos));
            }
            LoadingToLoaded => {
                log::debug!("Loaded chunk {:?}", pos);

                // We need to check, if chunk is within render distance, or outside load distance or still in load distance.
                let req = chunk_update_handler.create_state_update_request(pos, &world);
                state_update_req_ev.write(req);
            }
            LoadedToEmpty => {
                world.remove_chunk(pos);
                log::debug!("Removed chunk {:?}", pos);
            }
            LoadedToToDraw => {
                let chunk = chunk_update_handler.get_chunk_to_draw(pos, &mut world);
                chunk_build_req_ev.write(ChunkBuilderRequest::Build(pos, chunk));
            }
            ToDrawToLoaded => {
                chunk_build_req_ev.write(ChunkBuilderRequest::CancelBuilding(pos));
            }
            ToDrawToDrawn => {
                log::debug!("Built chunk {:?}", pos);

                let mesh = world.get_chunk_mesh(pos).unwrap();
                chunk_entity_req_ev.write(ChunkEntityEvent::Create(pos, mesh.clone()));
            }
            DrawnToToDraw => {
                let chunk = chunk_update_handler.get_chunk_to_draw(pos, &mut world);
                chunk_build_req_ev.write(ChunkBuilderRequest::Build(pos, chunk));
            }
            DrawnToLoaded => {
                world.chunk_meshes.remove(&pos);
                chunk_entity_req_ev.write(ChunkEntityEvent::Remove(pos));
            }
        }
    }
}
