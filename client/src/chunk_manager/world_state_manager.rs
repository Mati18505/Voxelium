use bevy::prelude::*;
use shared::entities::ChunkPos;
use shared::entities::ChunkRepository;

use crate::bevy_types::AppStates;
use crate::chunk_manager::events::*;
use crate::chunk_manager::resources::*;

pub struct WorldStateManagerPlugin;
impl Plugin for WorldStateManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicalWorldResource::default())
            .add_systems(Update, state_manager.run_if(in_state(AppStates::InGame)));
    }
}

// TODO
/*
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
 */

// TODO
/*
fn create_chunk_status(&self, pos: ChunkPos) -> ChunkStatus {
    let is_within_render = self.is_within_distance(pos, self.config.render_distance);
    let is_within_load = self.is_within_distance(pos, self.config.load_distance);
    let loaded = self.world.get_chunk(pos).is_some();
    let mesh_built = self.chunk_builder.is_chunk_with_latest_version_built(pos);
    let needs_rebuild = self.world.get_chunk_need_rebuild(pos);

    ChunkStatus {
        is_within_render,
        is_within_load,
        loaded,
        mesh_built,
        needs_rebuild,
    }
}
*/

// TODO: get those .clone() out
/// Adds loaded chunks and built meshes to world.
fn state_manager(
    mut world: ResMut<PhysicalWorldResource>,
    mut chunk_loaded_ev: EventReader<ChunkLoaded>,
    mut chunk_built_ev: EventReader<ChunkBuilt>,
    mut chunk_streamer_ev: EventReader<ChunkStreamerRequest>,
) {
    let world = &mut world.world;

    for ev in chunk_loaded_ev.read() {
        let pos = ev.chunk_pos;
        let chunk = ev.chunk.clone();

        world.set_chunk(pos, chunk);
        // TODO
        // update_chunk_state(pos);
    }

    for ev in chunk_built_ev.read() {
        let pos = ev.chunk_pos;
        let mesh = ev.chunk_mesh.clone();

        world.add_chunk_mesh(pos, mesh);
        // TODO
        // update_chunk_state(pos);
    }

    // TODO: remove only if state is `Empty`?
    /*
       for ev in chunk_streamer_ev.read() {
           match ev {
               ChunkStreamerRequest::Update(chunk_pos) => update_chunk_state(pos),
               ChunkStreamerRequest::Load(chunk_pos) => load_chunk_if_is_empty(pos),
               ChunkStreamerRequest::Remove(chunk_pos) => world.remove_chunk(pos),
           }
       }
    */
}
