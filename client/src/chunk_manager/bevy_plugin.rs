use std::sync::Arc;

use bevy::prelude::*;
use shared::{
    chunk_io::{
        providers::generated_chunk_provider::GeneratedChunkProvider, ChunkLoader, ChunkProvider,
    },
    entities::{BlockPos, Chunk, ChunkPos, ChunkPosGenerator2D, ChunkRepository},
};

use crate::{
    bevy_render::VoxelMaterial,
    bevy_types::{AppStates, GameResources},
    chunk_manager::{
        chunk_builder_system::{ChunkBuilderPlugin, ChunkBuilderResource}, chunk_data_manager::{chunk_loader_system::ChunkLoaderPlugin, world_event_handler::WorldEventHandlerPlugin}, chunk_entities_manager::{ChunkEntitiesManagerPlugin, ChunkEntitiesManagerResource}, chunk_loader_system::{ChunkLoaderPlugin, ChunkLoaderResource}, chunk_mesh_manager::{chunk_builder_system::ChunkBuilderPlugin, chunk_entities_manager::ChunkEntitiesManagerPlugin}, chunk_state_manager::ChunkStateManagerPlugin, chunk_streamer::{ChunkStreamerPlugin, StreamerConfig}, physical_world::PhysicalWorld, world_event_handler::WorldEventHandlerPlugin, ChunkState
    },
    chunk_mesh_builder::
        meshers::naive_mesher::NaiveMesher
    ,
    controller,
};

// TODO: change to plugin group
pub struct ChunkManagerPlugin;
impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChunkStreamerPlugin::new(StreamerConfig {
                load_distance: 10,
                render_distance: 9,
                dynamic_vertical_loading: false,
            }),
            ChunkBuilderPlugin,
            ChunkEntitiesManagerPlugin,
            ChunkLoaderPlugin,
            ChunkStateManagerPlugin,
            WorldEventHandlerPlugin,
        ))
        .add_systems(OnEnter(AppStates::InGame), init_manager)
        .add_systems(Update, update.run_if(in_state(AppStates::InGame)));
    }
}

fn init_manager(mut commands: Commands, game_resources: Res<GameResources>) {
    let chunk_mesher = Arc::new(NaiveMesher::new(
        game_resources.block_type_storage.clone(),
        game_resources.texture_dictionary.clone(),
    ));
    let chunk_provider = Box::new(GeneratedChunkProvider::new());
    let entities_manager_resource =
        ChunkEntitiesManagerResource::new(game_resources.opaque_texture.clone());
    let loader_resource = ChunkLoaderResource::new(chunk_provider);
    let builder_resource = ChunkBuilderResource::new(chunk_mesher);

    commands.insert_resource(entities_manager_resource);
    commands.insert_resource(loader_resource);
    commands.insert_resource(builder_resource);
}

fn update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    game_resources: Res<GameResources>,
    // mut chunk_manager_resources: ResMut<ChunkManagerResources>,
    mut voxel_materials: ResMut<Assets<VoxelMaterial>>,
    mut controller_events: EventReader<controller::PositionChangeEvent>,
    // chunk_manager_events: EventWriter<WorldChunkUpdateEvent>,
) {
    /*
          chunk_manager_resources
              .chunk_entities_manager
              .process_pending(
                  &mut commands,
                  &mut meshes,
                  game_resources.opaque_texture.clone(),
                  &mut voxel_materials,
              );
          chunk_manager_resources
              .event_manager
              .process_pending(chunk_manager_events);
    */
}
