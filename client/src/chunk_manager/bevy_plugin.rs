use std::sync::Arc;

use bevy::prelude::*;
use shared::chunk_io::providers::provider::ChunkProvider;
use shared::chunk_io::providers::terrain_generator::{TerrainConfig, TerrainGenerator};
use shared::entities::{BlockID, BlockInChunkPos, Chunk, ChunkRepository};
use shared::{
    chunk_io::providers::generated_chunk_provider::GeneratedChunkProvider,
    entities::{BlockPos, ChunkPos},
};

use crate::chunk_manager::bevy_chunk_entities_manager::{
    ChunkEntitiesPlugin, CreateEntity, RemoveEntity,
};
use crate::chunk_manager::{
    AddChunkMesh, ChunkBuilt, ChunkLoaded, ChunkLoaderPlugin, ChunkRemoved, ChunkStorage,
    ChunkStoragePlugin, ChunkUnloaded, DespawnChunk, RemoveChunkMesh, SpawnChunk,
};
use crate::chunk_mesh_builder::meshers::ChunkMesher;
use crate::{
    bevy_types::{AppStates, GameResources},
    chunk_manager::ChunkBuilderPlugin,
    chunk_mesh_builder::meshers::naive_mesher::NaiveMesher,
    controller,
};

#[derive(Message, Debug, PartialEq)]
pub struct VoxelEdit(pub BlockPos, pub BlockID);

#[derive(Message, Debug)]
pub struct ChunkUpdated(pub ChunkPos, pub Chunk);

pub struct ChunkManagerPlugin;
impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChunkLoaderPlugin,
            ChunkBuilderPlugin,
            ChunkEntitiesPlugin,
            ChunkStoragePlugin,
        ))
        .init_resource::<ControllerPos>()
        .init_resource::<TerrainGeneratorConfig>()
        .init_resource::<ChunkProviderResource>()
        .add_systems(OnEnter(AppStates::InGame), init_chunk_manager)
        .add_message::<ChunkUpdated>()
        .add_message::<VoxelEdit>()
        .add_systems(
            Update,
            (
                add_chunk_meshes,
                remove_chunk_meshes,
                create_chunk_entities,
                remove_chunk_entities,
                process_voxel_edits,
                handle_chunks_loaded,
                handle_chunks_unloaded,
            )
                .run_if(in_state(AppStates::InGame)),
        )
        .add_systems(
            Update,
            reload_terrain_generator.run_if(resource_changed::<TerrainGeneratorConfig>),
        )
        .add_observer(on_position_change);
    }
}

#[derive(Resource)]
pub struct ControllerPos(pub ChunkPos);

impl Default for ControllerPos {
    fn default() -> Self {
        Self(ChunkPos::new(0, 0, 0))
    }
}

#[derive(Resource, Default)]
pub struct TerrainGeneratorConfig(pub TerrainConfig);

#[derive(Resource)]
pub struct ChunkMesherResource(pub Arc<dyn ChunkMesher>);
#[derive(Resource)]
pub struct ChunkProviderResource(pub Box<dyn ChunkProvider>);

impl Default for ChunkProviderResource {
    fn default() -> Self {
        let chunk_generator = Box::new(GeneratedChunkProvider::default());
        Self(chunk_generator)
    }
}

fn init_chunk_manager(mut commands: Commands, game_resources: Res<GameResources>) {
    let voxel_mesher = NaiveMesher::new((*game_resources.render_shape_storage).clone());
    let voxel_mesher = Arc::new(voxel_mesher);
    commands.insert_resource(ChunkMesherResource(voxel_mesher));
}

fn reload_terrain_generator(
    mut provider: ResMut<ChunkProviderResource>,
    config: Res<TerrainGeneratorConfig>,
) {
    let chunk_generator = Box::new(GeneratedChunkProvider::new(TerrainGenerator::new(config.0)));
    *provider = ChunkProviderResource(chunk_generator);
}

fn handle_chunks_loaded(
    mut loaded_chunks: MessageReader<ChunkLoaded>,
    mut chunks_to_spawn: MessageWriter<SpawnChunk>,
) {
    for loaded in loaded_chunks.read() {
        let ChunkLoaded(pos, chunk) = loaded.clone();

        chunks_to_spawn.write(SpawnChunk(pos, chunk));
    }
}

fn handle_chunks_unloaded(
    mut unloaded_chunks: MessageReader<ChunkUnloaded>,
    mut chunks_to_despawn: MessageWriter<DespawnChunk>,
) {
    for unloaded in unloaded_chunks.read() {
        let ChunkUnloaded(pos) = unloaded;

        chunks_to_despawn.write(DespawnChunk(*pos));
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

fn add_chunk_meshes(
    mut built_chunks: MessageReader<ChunkBuilt>,
    mut add_chunk_meshes: MessageWriter<AddChunkMesh>,
) {
    for ChunkBuilt(pos, mesh) in built_chunks.read().cloned() {
        add_chunk_meshes.write(AddChunkMesh(pos, mesh));
    }
}

fn remove_chunk_meshes(
    mut removed_chunks: MessageReader<ChunkRemoved>,
    mut remove_chunk_meshes: MessageWriter<RemoveChunkMesh>,
) {
    for ChunkRemoved(pos) in removed_chunks.read().cloned() {
        remove_chunk_meshes.write(RemoveChunkMesh(pos));
    }
}

fn process_voxel_edits(
    mut voxel_edits: MessageReader<VoxelEdit>,
    mut chunk_updated: MessageWriter<ChunkUpdated>,
    mut chunks: ChunkStorage,
) {
    for edit in voxel_edits.read() {
        let block_pos = edit.0;
        let new_voxel = edit.1;

        let (chunk_pos, block_in_chunk_pos) =
            (ChunkPos::from(block_pos), BlockInChunkPos::from(block_pos));

        let Some(chunk) = chunks.get_chunk_mut(chunk_pos) else {
            warn!("chunk not found for voxel edit {:?}", edit);
            continue;
        };

        let mut new_block_storage = chunk.get_block_storage().clone();

        new_block_storage.set_block(block_in_chunk_pos, new_voxel);
        *chunk = Chunk::new(new_block_storage);

        chunk_updated.write(ChunkUpdated(chunk_pos, chunk.clone()));
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
