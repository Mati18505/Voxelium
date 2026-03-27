use std::sync::Arc;

use bevy::prelude::*;
use shared::chunk_io::providers::provider::ChunkProvider;
use shared::entities::{BlockID, BlockInChunkPos, Chunk, ChunkRepository};
use shared::{
    chunk_io::providers::generated_chunk_provider::GeneratedChunkProvider,
    entities::{BlockPos, ChunkPos},
};

use crate::chunk_manager::bevy_chunk_entities_manager::{
    ChunkEntitiesPlugin, CreateEntity, RemoveEntity,
};
use crate::chunk_manager::{
    ChunkBuilt, ChunkLoaded, ChunkLoaderConfig, ChunkLoaderPlugin, ChunkRemoved, ChunkUnloaded,
};
use crate::chunk_mesh_builder::meshers::ChunkMesher;
use crate::{
    bevy_types::{AppStates, GameResources},
    chunk_manager::ChunkBuilderPlugin,
    chunk_mesh_builder::meshers::naive_mesher::NaiveMesher,
    controller,
};

use super::ChunkBuilderConfig;

#[derive(Message, Debug, PartialEq)]
pub struct VoxelEdit(pub BlockPos, pub BlockID);

#[derive(Message, Debug)]
pub struct ChunkUpdated(pub ChunkPos, pub Chunk);

pub struct ChunkManagerPlugin;
impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChunkLoaderPlugin::new(ChunkLoaderConfig {
                max_loads_per_frame: 64,
                load_distance: 81,
                dynamic_vertical_loading: false,
                debug: false,
            }),
            ChunkBuilderPlugin::new(ChunkBuilderConfig {
                max_builds_per_frame: 64,
                render_distance: 80,
                dynamic_vertical_loading: false,
                debug: false,
            }),
            ChunkEntitiesPlugin,
        ))
        .init_resource::<ChunkStorage>()
        .init_resource::<ControllerPos>()
        .add_systems(OnEnter(AppStates::InGame), init_chunk_manager)
        .add_message::<ChunkUpdated>()
        .add_message::<VoxelEdit>()
        .add_systems(
            Update,
            (
                insert_chunks,
                remove_chunks,
                create_chunk_entities,
                remove_chunk_entities,
                process_voxel_edits,
            )
                .run_if(in_state(AppStates::InGame)),
        )
        .add_observer(on_position_change);
    }
}

#[derive(Resource, Default)]
pub struct ChunkStorage(pub shared::entities::World);

#[derive(Resource)]
pub struct ControllerPos(pub ChunkPos);

impl Default for ControllerPos {
    fn default() -> Self {
        Self(ChunkPos::new(0, 0, 0))
    }
}

#[derive(Resource)]
pub struct ChunkMesherResource(pub Arc<dyn ChunkMesher>);
#[derive(Resource)]
pub struct ChunkProviderResource(pub Box<dyn ChunkProvider>);

fn init_chunk_manager(mut commands: Commands, game_resources: Res<GameResources>) {
    let chunk_generator = Box::new(GeneratedChunkProvider::new());
    commands.insert_resource(ChunkProviderResource(chunk_generator));

    let voxel_mesher = NaiveMesher::new((*game_resources.render_shape_storage).clone());
    let voxel_mesher = Arc::new(voxel_mesher);
    commands.insert_resource(ChunkMesherResource(voxel_mesher));
}

fn insert_chunks(mut loaded_chunks: MessageReader<ChunkLoaded>, mut chunks: ResMut<ChunkStorage>) {
    for loaded in loaded_chunks.read() {
        let pos = loaded.0;
        let chunk = loaded.1.clone();

        chunks.0.set_chunk(pos, chunk);
    }
}

fn remove_chunks(
    mut unloaded_chunks: MessageReader<ChunkUnloaded>,
    mut chunks: ResMut<ChunkStorage>,
) {
    for unloaded in unloaded_chunks.read() {
        let pos = unloaded.0;

        chunks.0.remove_chunk(pos);
    }
}

fn create_chunk_entities(
    mut built_chunks: MessageReader<ChunkBuilt>,
    mut request: MessageWriter<CreateEntity>,
) {
    for built in built_chunks.read() {
        let pos = built.0;
        let mesh = built.1.clone();

        request.write(CreateEntity(pos, mesh));
    }
}

fn remove_chunk_entities(
    mut removed_chunks: MessageReader<ChunkRemoved>,
    mut request: MessageWriter<RemoveEntity>,
) {
    for removed in removed_chunks.read() {
        let pos = removed.0;

        request.write(RemoveEntity(pos));
    }
}

fn process_voxel_edits(
    mut voxel_edits: MessageReader<VoxelEdit>,
    mut chunk_updated: MessageWriter<ChunkUpdated>,
    mut data: ResMut<ChunkStorage>,
) {
    for edit in voxel_edits.read() {
        let block_pos = edit.0;
        let new_voxel = edit.1;

        let (chunk_pos, block_in_chunk_pos) =
            (ChunkPos::from(block_pos), BlockInChunkPos::from(block_pos));

        let Some(chunk) = data.0.get_chunk(chunk_pos) else {
            warn!("chunk not found for voxel edit {:?}", edit);
            continue;
        };

        let mut new_block_storage = chunk.get_block_storage().clone();

        new_block_storage.set_block(block_in_chunk_pos, new_voxel);
        let new_chunk = Chunk::new(new_block_storage);

        data.0.set_chunk(chunk_pos, new_chunk.clone());

        chunk_updated.write(ChunkUpdated(chunk_pos, new_chunk));
    }
}

fn on_position_change(
    e: On<controller::PositionChangeEvent>,
    state: Res<State<AppStates>>,
    mut controller_pos: ResMut<ControllerPos>,
) {
    if !matches!(state.get(), AppStates::InGame) {
        return;
    }

    let new_pos = e.new_pos;
    let new_block_pos = BlockPos::new(new_pos.x as isize, new_pos.y as isize, new_pos.z as isize);
    let new_chunk_pos = ChunkPos::from(new_block_pos);

    if controller_pos.0 != new_chunk_pos {
        controller_pos.0 = new_chunk_pos;
    }
}
