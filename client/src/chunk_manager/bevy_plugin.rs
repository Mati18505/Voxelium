use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use bevy::prelude::*;
use shared::{
    chunk_io::{providers::generated_chunk_provider::GeneratedChunkProvider, ChunkLoader},
    entities::{BlockPos, ChunkPos},
};

use crate::chunk_manager::ChunkBuilt;
use crate::chunk_mesh_builder::meshers::ChunkMesher;
use crate::{
    bevy_types::{AppStates, GameResources},
    chunk_manager::{
        BuildChunk, ChunkBuilderPlugin, ChunkObjectEvent, RemoveChunk, WorldChunkUpdate,
    },
    chunk_mesh_builder::meshers::naive_mesher::NaiveMesher,
    controller,
};

use super::ChunkBuilderConfig;
use super::{
    bevy_event_manager::WorldChunkUpdateEvent, ChunkEntitiesManager, ChunkManager, Config,
    EventManager,
};

pub struct ChunkManagerPlugin;
impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkBuilderPlugin::new(ChunkBuilderConfig {
            max_build_tasks: 16,
        }))
        .init_resource::<ChunkStorage>()
        .init_resource::<ControllerPos>()
        .add_systems(OnEnter(AppStates::InGame), init_chunk_manager)
        .add_message::<WorldChunkUpdateEvent>()
        .add_systems(Update, update.run_if(in_state(AppStates::InGame)))
        .add_observer(on_position_change);
    }
}

#[derive(Resource)]
pub struct ChunkManagerResources {
    pub chunk_manager: ChunkManager,
    chunk_entities_manager: ChunkEntitiesManager,
    event_manager: EventManager,
}

#[derive(Resource, Default)]
pub struct ChunkStorage(pub shared::entities::World);

impl Deref for ChunkStorage {
    type Target = shared::entities::World;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChunkStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Resource)]
pub struct ControllerPos(pub ChunkPos);

impl Default for ControllerPos {
    fn default() -> Self {
        Self(ChunkPos::new(0, 0, 0))
    }
}

impl Deref for ControllerPos {
    type Target = ChunkPos;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ControllerPos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Resource)]
pub struct ChunkMesherResource(pub Arc<dyn ChunkMesher>);

fn init_chunk_manager(mut commands: Commands, game_resources: Res<GameResources>) {
    let mut config = Config::new(20, 19);
    config.dynamic_vertical_loading = false;

    let (chunk_object_tx, chunk_object_rx) = crossbeam_channel::unbounded::<ChunkObjectEvent>();
    let (event_tx, event_rx) = crossbeam_channel::unbounded::<WorldChunkUpdate>();
    let chunk_loader_provider = Box::new(GeneratedChunkProvider::new());
    let chunk_loader = ChunkLoader::new(chunk_loader_provider);

    let mut chunk_manager = ChunkManager::new(chunk_loader, config);

    chunk_manager.set_chunk_object_tx(Some(chunk_object_tx));
    chunk_manager.set_event_tx(Some(event_tx));

    let chunk_manager_resources = ChunkManagerResources {
        chunk_manager,
        chunk_entities_manager: ChunkEntitiesManager::new(chunk_object_rx),
        event_manager: EventManager::new(event_rx),
    };

    commands.insert_resource(chunk_manager_resources);

    let voxel_mesher = NaiveMesher::new((*game_resources.render_shape_storage).clone());
    let voxel_mesher = Arc::new(voxel_mesher);
    commands.insert_resource(ChunkMesherResource(voxel_mesher));
}

fn update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    game_resources: Res<GameResources>,
    mut chunk_manager_resources: ResMut<ChunkManagerResources>,
    chunk_manager_events: MessageWriter<WorldChunkUpdateEvent>,
    mut chunks_to_build: MessageWriter<BuildChunk>,
    mut chunks_to_remove: MessageWriter<RemoveChunk>,
    mut chunks: ResMut<ChunkStorage>,
    controller_pos: Res<ControllerPos>,
    mut built_chunks: MessageReader<ChunkBuilt>,
) {
    chunk_manager_resources
        .chunk_manager
        .check_loaded_chunks(&mut chunks, &controller_pos);
    chunk_manager_resources.chunk_manager.update_built_chunks(
        built_chunks,
        &mut chunks,
        &controller_pos,
    );
    chunk_manager_resources
        .chunk_entities_manager
        .process_pending(
            &mut commands,
            &mut meshes,
            game_resources.material_storage.clone(),
        );
    chunk_manager_resources
        .event_manager
        .process_pending(chunk_manager_events);

    chunk_manager_resources
        .chunk_manager
        .send_messages_to_builder(&mut chunks_to_build, &mut chunks_to_remove);
}

fn on_position_change(
    e: On<controller::PositionChangeEvent>,
    mut chunk_manager_resources: Option<ResMut<ChunkManagerResources>>,
    state: Res<State<AppStates>>,
    mut chunks: ResMut<ChunkStorage>,
    controller_pos: ResMut<ControllerPos>,
) {
    if !matches!(state.get(), AppStates::InGame) {
        return;
    }

    let new_pos = e.new_pos;
    let new_block_pos = BlockPos::new(new_pos.x as isize, new_pos.y as isize, new_pos.z as isize);
    let new_chunk_pos = ChunkPos::from(new_block_pos);

    if let Some(chunk_manager_resources) = &mut chunk_manager_resources {
        chunk_manager_resources.chunk_manager.update_controller_pos(
            controller_pos,
            new_chunk_pos,
            &mut chunks,
        );
    } else {
        warn!("chunk_manager_resources is null in on_position_change");
    }
}
