use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use shared::{chunk_loader::ChunkLoader, entities::{BlockPos, ChunkPos}};

use crate::{bevy_render::VoxelMaterial, bevy_types::{AppStates, GameResources}, chunk_manager::bevy_event_manager::WorldChunkUpdateEvent, controller};

use super::{ChunkManager, ChunkEntitiesManager, Config, AsyncChunkBuilder, EventManager};

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
    chunk_entities_manager: Arc<Mutex<ChunkEntitiesManager>>,
    event_manager: Arc<Mutex<EventManager>>,
}

fn init_chunk_manager(
    mut commands: Commands,
    game_resources: Res<GameResources>,
) {
    let chunk_builder = Box::new(AsyncChunkBuilder::new(game_resources.block_type_storage.clone(), game_resources.texture_dictionary.clone()));
    let mut config = Config::new(5, 4);
    config.dynamic_vertical_loading = true;

    let chunk_entities_manager = Arc::new(Mutex::new(ChunkEntitiesManager::default()));
    let event_manager = Arc::new(Mutex::new(EventManager::default()));
    let mut chunk_manager = ChunkManager::new(ChunkLoader::default(), chunk_builder, config);

    chunk_manager.set_chunk_object_callback(chunk_entities_manager.clone());
    chunk_manager.set_event_callback(event_manager.clone());

    let chunk_manager_resources = ChunkManagerResources {
        chunk_manager,
        chunk_entities_manager,
        event_manager,
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
    }

    chunk_manager_resources.chunk_manager.check_builded_chunks();

    if let Ok(mut manager) = chunk_manager_resources.chunk_entities_manager.lock() {
        manager.process_pending(&mut commands, &mut meshes, game_resources.opaque_texture.clone(), &mut voxel_materials);
    }

    if let Ok(mut event_manager) = chunk_manager_resources.event_manager.lock() {
        event_manager.process_pending(chunk_manager_events);
    }
}