use std::sync::Arc;

use bevy::prelude::*;
use shared::{
    chunk_io::{
        providers::generated_chunk_provider::GeneratedChunkProvider, ChunkLoader, ChunkProvider,
    },
    entities::{BlockPos, Chunk, ChunkPos, ChunkPosGenerator2D, ChunkRepository},
};

use crate::{
    bevy_types::{AppStates, GameResources},
    chunk_manager::{
        chunk_builder_system::{ChunkBuilderPlugin, ChunkBuilderResource},
        chunk_entities_manager::{ChunkEntitiesManagerPlugin, ChunkEntitiesManagerResource},
        chunk_loader_system::{ChunkLoaderPlugin, ChunkLoaderResource},
        chunk_state_manager::ChunkStateManagerPlugin,
        chunk_streamer::{ChunkStreamerPlugin, StreamerConfig},
        physical_world::PhysicalWorld,
        world_event_handler::WorldEventHandlerPlugin,
        ChunkState,
    },
    chunk_mesh_builder::{
        builders::{self, async_chunk_builder::AsyncChunkBuilder, ChunkBuilder, Versioned},
        meshers::{naive_mesher::NaiveMesher, ChunkMesher},
        ChunkMesh,
    },
    controller,
    orchestrator::ChunkPosChangedEvent,
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
        .add_systems(Update, update.run_if(in_state(AppStates::InGame)))
        .add_message::<WorldChunkUpdateEvent>()
        .add_observer(on_position_change);
    }
}

fn init_manager(mut commands: Commands, game_resources: Res<GameResources>) {
    let voxel_mesher = NaiveMesher::new((*game_resources.render_shape_storage).clone());
    let chunk_provider = Box::new(GeneratedChunkProvider::new());
    let entities_manager_resource =
        ChunkEntitiesManagerResource::new(game_resources.opaque_texture.clone());
    let loader_resource = ChunkLoaderResource::new(chunk_provider);
    let builder_resource = ChunkBuilderResource::new(voxel_mesher);

    commands.insert_resource(entities_manager_resource);
    commands.insert_resource(loader_resource);
    commands.insert_resource(builder_resource);
}

fn update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    game_resources: Res<GameResources>,
    mut chunk_manager_resources: ResMut<ChunkManagerResources>,
    mut controller_events: EventReader<controller::PositionChangeEvent>,
    chunk_manager_events: EventWriter<WorldChunkUpdateEvent>,
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

fn on_position_change(
    e: On<controller::PositionChangeEvent>,
    mut chunk_manager_resources: Option<ResMut<ChunkManagerResources>>,
    state: Res<State<AppStates>>,
) {
    if !matches!(state.get(), AppStates::InGame) {
        return;
    }

    let new_pos = e.new_pos;
    let new_block_pos = BlockPos::new(new_pos.x as isize, new_pos.y as isize, new_pos.z as isize);
    let new_chunk_pos = ChunkPos::from(new_block_pos);

    if let Some(chunk_manager_resources) = &mut chunk_manager_resources {
        chunk_manager_resources
            .chunk_manager
            .update_controller_pos(new_chunk_pos);
    } else {
        warn!("chunk_manager_resources is null in on_position_change");
    }

    for e in controller_events.read() {
        let new_pos = e.new_pos;
        dbg!(&new_pos);
        let new_block_pos =
            BlockPos::new(new_pos.x as isize, new_pos.y as isize, new_pos.z as isize);
        let new_chunk_pos = ChunkPos::from(new_block_pos);

        chunk_manager_resources
            .chunk_manager
            .update_controller_pos(new_chunk_pos);
        // dbg!(&chunk_manager_resources.chunk_manager);
    }

    chunk_manager_resources.chunk_manager.check_loaded_chunks();
    chunk_manager_resources.chunk_manager.check_built_chunks();
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
}

fn on_position_change(
    e: On<controller::PositionChangeEvent>,
    mut chunk_manager_resources: Option<ResMut<ChunkManagerResources>>,
    state: Res<State<AppStates>>,
) {
    if !matches!(state.get(), AppStates::InGame) {
        return;
    }

    let new_pos = e.new_pos;
    let new_block_pos = BlockPos::new(new_pos.x as isize, new_pos.y as isize, new_pos.z as isize);
    let new_chunk_pos = ChunkPos::from(new_block_pos);

    if let Some(chunk_manager_resources) = &mut chunk_manager_resources {
        chunk_manager_resources
            .chunk_manager
            .update_controller_pos(new_chunk_pos);
    } else {
        warn!("chunk_manager_resources is null in on_position_change");
    }
}
