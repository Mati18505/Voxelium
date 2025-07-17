use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use shared::{chunk_io::{providers::generated_chunk_provider::GeneratedChunkProvider, ChunkLoader}, entities::{BlockPos, ChunkPos}};

use crate::{bevy_render::VoxelMaterial, bevy_types::{AppStates, GameResources}, chunk_mesh_builder::VoxelMesher, chunk_state_manager::{ChunkObjectEvent, WorldChunkUpdate}, controller};

use super::{bevy_event_manager::WorldChunkUpdateEvent, AsyncChunkBuilder, ChunkEntitiesManager, ChunkManager, Config, EventManager};

pub struct ChunkManagerPlugin;
impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppStates::InGame), init_chunk_manager)
        .add_event::<WorldChunkUpdateEvent>()
        .add_systems(Update, update.run_if(in_state(AppStates::InGame)));
    }
}

#[derive(Resource)]
pub struct ChunkManagerResources {
    pub chunk_manager: ChunkManager,
    chunk_entities_manager: ChunkEntitiesManager,
    event_manager: EventManager,
}

fn init_chunk_manager(
    mut commands: Commands,
    game_resources: Res<GameResources>,
) {
    let voxel_mesher = VoxelMesher::new(
        game_resources.block_type_storage.clone(),
        game_resources.texture_dictionary.clone(),
    );

    let chunk_builder = Box::new(AsyncChunkBuilder::new(voxel_mesher));
    let mut config = Config::new(10, 9);
    config.dynamic_vertical_loading = false;

    let (chunk_object_tx, chunk_object_rx) = crossbeam_channel::unbounded::<ChunkObjectEvent>();
    let (event_tx, event_rx) = crossbeam_channel::unbounded::<WorldChunkUpdate>();
    let chunk_loader_provider = Box::new(GeneratedChunkProvider::new());
    let chunk_loader = ChunkLoader::new(chunk_loader_provider);

    let mut chunk_manager = ChunkManager::new(chunk_loader, chunk_builder, config);

    chunk_manager.set_chunk_object_tx(Some(chunk_object_tx));
    chunk_manager.set_event_tx(Some(event_tx));

    let chunk_manager_resources = ChunkManagerResources {
        chunk_manager,
        chunk_entities_manager: ChunkEntitiesManager::new(chunk_object_rx),
        event_manager: EventManager::new(event_rx),
    };
    commands.insert_resource(chunk_manager_resources);
}

fn update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    game_resources: Res<GameResources>,
    mut chunk_manager_resources: ResMut<ChunkManagerResources>,
    mut voxel_materials: ResMut<Assets<VoxelMaterial>>,
    mut controller_events: EventReader<controller::PositionChangeEvent>,
    chunk_manager_events: EventWriter<WorldChunkUpdateEvent>,
) {
    for e in controller_events.read() {
        let new_pos = e.new_pos;
        // Convert bevy direction to our direction
        let new_block_pos = BlockPos::new(new_pos.x as isize, -new_pos.z as isize, new_pos.y as isize);
        let new_chunk_pos = ChunkPos::from(new_block_pos);

        chunk_manager_resources.chunk_manager.update_controller_pos(new_chunk_pos);
        dbg!(&chunk_manager_resources.chunk_manager);
    }

    chunk_manager_resources.chunk_manager.check_loaded_chunks();
    chunk_manager_resources.chunk_manager.check_built_chunks();
    chunk_manager_resources.chunk_entities_manager.process_pending(&mut commands, &mut meshes, game_resources.opaque_texture.clone(), &mut voxel_materials);
    chunk_manager_resources.event_manager.process_pending(chunk_manager_events);
}