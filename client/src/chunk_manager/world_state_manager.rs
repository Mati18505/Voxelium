use bevy::prelude::*;
use shared::entities::ChunkPos;
use shared::entities::ChunkRepository;

use crate::bevy_types::AppStates;
use crate::chunk_manager::chunk_streamer::StreamerConfig;
use crate::chunk_manager::events::*;
use crate::chunk_manager::physical_world::PhysicalWorld;
use crate::chunk_manager::resources::*;
use crate::chunk_manager::ChunkStatus;

pub struct WorldStateManagerPlugin;
impl Plugin for WorldStateManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicalWorldResource::default())
            .add_systems(Update, state_manager.run_if(in_state(AppStates::InGame)));
    }
}

#[derive(Event, Debug)]
struct StateUpdateRequest {
    pub chunk_status: ChunkStatus,
}

fn is_within_distance(
    controller_pos: ChunkPos,
    config: &Config,
    pos: ChunkPos,
    distance_in_chunks: usize,
) -> bool {
    match config.dynamic_vertical_loading {
        true => pos.is_within_distance(controller_pos, distance_in_chunks),
        false => pos.is_within_distance_2d(controller_pos, distance_in_chunks),
    }
}

fn create_chunk_status(
    pos: ChunkPos,
    controller_pos: ChunkPos,
    config: &StreamerConfig,
    world: &PhysicalWorld,
) -> ChunkStatus {
    let is_within_render = is_within_distance(controller_pos, config, pos, config.render_distance);
    let is_within_load = is_within_distance(controller_pos, config, pos, config.load_distance);
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

// TODO: get those .clone() out
/// Adds loaded chunks and built meshes to world.
fn state_manager(
    mut world: ResMut<PhysicalWorldResource>,
    mut state_update_req_ev: EventWriter<StateUpdateRequest>,
    mut chunk_loaded_ev: EventReader<ChunkLoaded>,
    mut chunk_built_ev: EventReader<ChunkBuilt>,
    mut chunk_streamer_ev: EventReader<ChunkStreamerRequest>,
    config: Res<StreamerConfig>,
    chunk_manager_resources: Res<ChunkManagerResource>,
) {
    let world = &mut world.world;
    let controller_pos = chunk_manager_resources.controller_pos;

    let build_chunk_status = |pos| create_chunk_status(pos, controller_pos, config, world);
    let update_chunk_state = |pos| {
        let chunk_status = build_chunk_status(pos);
        state_update_req_ev.write(chunk_status);
    };

    for ev in chunk_loaded_ev.read() {
        let pos = ev.chunk_pos;
        let chunk = ev.chunk.clone();

        world.set_chunk(pos, chunk);

        update_chunk_state(pos);
    }

    for ev in chunk_built_ev.read() {
        let pos = ev.chunk_pos;
        let mesh = ev.chunk_mesh.clone();

        world.add_chunk_mesh(pos, mesh);

        update_chunk_state(pos);
    }

    // TODO: remove only if state is `Empty`?
    for ev in chunk_streamer_ev.read() {
        match ev {
            ChunkStreamerRequest::Update(pos) => update_chunk_state(pos),
            ChunkStreamerRequest::Remove(pos) => world.remove_chunk(pos),
            ChunkStreamerRequest::Load(pos) => {
                let mut chunk_status = build_chunk_status(pos);
                chunk_status.is_within_load = true;
                state_update_req_ev.write(chunk_status);
            },
        }
    }
}
