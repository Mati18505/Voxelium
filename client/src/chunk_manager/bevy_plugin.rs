use std::sync::Arc;

use bevy::prelude::*;
use shared::{
    chunk_io::{
        providers::generated_chunk_provider::GeneratedChunkProvider, ChunkLoader, ChunkProvider,
    },
    entities::{BlockPos, ChunkPos},
};

use crate::{
    bevy_render::VoxelMaterial,
    bevy_types::{AppStates, GameResources},
    chunk_manager::{physical_world::PhysicalWorld, ChunkManagerSystemsPlugin},
    chunk_mesh_builder::{
        builders::{async_chunk_builder::AsyncChunkBuilder, ChunkBuilder, Versioned},
        meshers::{naive_mesher::NaiveMesher, ChunkMesher},
    },
    controller,
    orchestrator::ChunkPosChangedEvent,
};

use super::{bevy_event_manager::WorldChunkUpdateEvent, ChunkEntitiesManager};

pub struct ChunkManagerPlugin;
impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppStates::InGame), init__manager)
            .add_event::<WorldChunkUpdateEvent>()
            .add_systems(Update, update.run_if(in_state(AppStates::InGame)))
            .add_systems(Update, chunk_streamer.run_if(in_state(AppStates::InGame)))
            .insert_resource(ChunkManagerResources::default());
    }
}

#[derive(Resource)]
pub struct ManagerResources {
    chunk_entities_manager: ChunkEntitiesManager,
}

fn init_manager(mut commands: Commands, game_resources: Res<GameResources>) {
    let voxel_mesher = NaiveMesher::new(
        game_resources.block_type_storage.clone(),
        game_resources.texture_dictionary.clone(),
    );

    let mut config = Config::new(10, 9);
    config.dynamic_vertical_loading = false;

    let chunk_provider = Box::new(GeneratedChunkProvider::new());

    let chunk_manager_resource = ChunkManagerResources::new(voxel_mesher, chunk_provider, config);

    commands.insert_resource(resource);

    let manager_resources = ManagerResources {
        chunk_entities_manager: ChunkEntitiesManager::new(chunk_object_rx),
    };

    commands.insert_resource(manager_resources);
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

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Horizontal radius (in chunks) within which chunks are loaded.
    pub load_distance: usize,
    /// Horizontal radius (in chunks) within which chunks are rendered.
    pub render_distance: usize,
    /// If true, the engine dynamically loads chunks above and below the player based on vertical position.
    pub dynamic_vertical_loading: bool,
}

impl Config {
    pub fn new(load_distance: usize, render_distance: usize) -> Self {
        assert!(render_distance <= load_distance);

        Config {
            load_distance,
            render_distance,
            dynamic_vertical_loading: false,
        }
    }
}

pub trait VersionedChunkBuilder<T: Send + Sync + Default>: ChunkBuilder<T> + Versioned<T> {}

impl<T: Send + Sync + Default, U> VersionedChunkBuilder<T> for U where
    U: ChunkBuilder<T> + Versioned<T>
{
}

#[derive(Resource)]
pub struct ChunkManagerResources {
    world: PhysicalWorld,
    chunk_loader: ChunkLoader,
    chunk_builder: Box<dyn VersionedChunkBuilder<()>>,
    config: Config,
}

impl ChunkManagerResources {
    fn new(
        voxel_mesher: Arc<dyn ChunkMesher>,
        chunk_provider: Box<dyn ChunkProvider>,
        config: Config,
    ) {
        let chunk_loader = ChunkLoader::new(chunk_provider);

        let inner_builder = Box::new(AsyncChunkBuilder::new(Arc::new(voxel_mesher)));
        let chunk_builder = Box::new(VersionedChunkBuilder::<()>::new(inner_builder));

        Self {
            world: PhysicalWorld::default(),
            chunk_loader,
            chunk_builder,
            config,
        }
    }
}

fn chunk_streamer(chunk_pos_changed_ev: EventWriter<ChunkPosChangedEvent>) {
    println!("3");
    for ev in chunk_pos_changed_ev.read() {
        println!("chunk pos: {}");
    }
}
