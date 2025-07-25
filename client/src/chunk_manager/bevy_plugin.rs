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
    chunk_manager::chunk_data_manager::{
        chunk_loader_system::{ChunkLoaderPlugin, ChunkLoaderResource},
        chunk_streamer::{ChunkStreamerPlugin, StreamerConfig},
        world_event_handler::{WorldEventHandlerConfig, WorldEventHandlerPlugin},
    },
    chunk_mesh_builder::meshers::naive_mesher::NaiveMesher,
    controller,
};

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct ChunkManagerConfig {
    /// Horizontal radius (in chunks) within which chunks are loaded.
    pub load_distance: usize,
    /// Horizontal radius (in chunks) within which chunks are rendered.
    pub render_distance: usize,
    /// If true, the engine dynamically loads chunks above and below the player based on vertical position.
    pub dynamic_vertical_loading: bool,
}
impl ChunkManagerConfig {
    pub fn new(load_distance: usize, render_distance: usize) -> Self {
        assert!(render_distance <= load_distance);

        ChunkManagerConfig {
            load_distance,
            render_distance,
            dynamic_vertical_loading: false,
        }
    }
}

// TODO: change to plugin group
pub struct ChunkManagerPlugin {
    config: ChunkManagerConfig,
}
impl ChunkManagerPlugin {
    pub fn new(config: ChunkManagerConfig) -> Self {
        Self { config }
    }
}

impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ChunkStreamerPlugin::new(StreamerConfig {
                load_distance: self.config.load_distance,
                dynamic_vertical_loading: self.config.dynamic_vertical_loading,
            }),
            ChunkLoaderPlugin,
            WorldEventHandlerPlugin::new(WorldEventHandlerConfig {
                load_distance: self.config.load_distance,
                dynamic_vertical_loading: self.config.dynamic_vertical_loading,
            }),
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
    // let entities_manager_resource =
    //     ChunkEntitiesManagerResource::new(game_resources.opaque_texture.clone());
    let loader_resource = ChunkLoaderResource::new(chunk_provider);
    // let builder_resource = ChunkBuilderResource::new(chunk_mesher);

    // commands.insert_resource(entities_manager_resource);
    commands.insert_resource(loader_resource);
    // commands.insert_resource(builder_resource);
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
