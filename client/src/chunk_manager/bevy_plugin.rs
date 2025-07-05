use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use shared::{chunk_loader::ChunkLoader, entities::{BlockPos, ChunkPos}};

use crate::{bevy_render::VoxelMaterial, bevy_types::{AppStates, GameResources}, chunk_builder::{BlockTypeStorage, TextureDictionary}, controller};

use super::{ChunkManager, ChunkEntitiesManager, Config, AsyncChunkBuilder};

pub struct ChunkManagerPlugin;
impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppStates::InGame), init_chunk_manager)
        .add_systems(Update, update.run_if(in_state(AppStates::InGame)));
    }
}

#[derive(Resource)]
pub struct ChunkManagerResources {
    chunk_manager: ChunkManager,
    chunk_entities_manager: Arc<Mutex<ChunkEntitiesManager>>,
}

fn init_chunk_manager(
    mut commands: Commands,
    game_resources: Res<GameResources>,
) {
    let chunk_builder = Box::new(AsyncChunkBuilder::new(game_resources.block_type_storage.clone(), game_resources.texture_dictionary.clone()));
    let config = Config::new(17, 16);

    let chunk_entities_manager = Arc::new(Mutex::new(ChunkEntitiesManager::default()));
    let mut chunk_manager = ChunkManager::new(ChunkLoader::default(), chunk_builder, config);
    chunk_manager.set_chunk_object_callback(chunk_entities_manager.clone());

    let chunk_manager_resources = ChunkManagerResources {
        chunk_manager,
        chunk_entities_manager,
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
}