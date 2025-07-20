use std::sync::Arc;

use bevy::prelude::*;
use shared::{
    chunk_io::{
        providers::generated_chunk_provider::GeneratedChunkProvider, ChunkLoader, ChunkProvider,
    },
    entities::{BlockPos, ChunkPos, ChunkPosGenerator2D, ChunkRepository},
};

use crate::{
    bevy_render::VoxelMaterial,
    bevy_types::{AppStates, GameResources},
    chunk_manager::{physical_world::PhysicalWorld},
    chunk_mesh_builder::{
        builders::{self, async_chunk_builder::AsyncChunkBuilder, ChunkBuilder, Versioned},
        meshers::{naive_mesher::NaiveMesher, ChunkMesher},
    },
    controller,
    orchestrator::ChunkPosChangedEvent,
};

use super::{bevy_event_manager::WorldChunkUpdateEvent, ChunkEntitiesManager};

pub struct ChunkManagerPlugin;
impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Config{
            load_distance: 10,
            render_distance: 9,
            dynamic_vertical_loading: false,
        })
        .add_systems(OnEnter(AppStates::InGame), init_manager)
            .add_event::<WorldChunkUpdateEvent>()
            .add_systems(Update, update.run_if(in_state(AppStates::InGame)))
            .add_systems(Update, chunk_streamer.run_if(in_state(AppStates::InGame)));
    }
}

#[derive(Resource)]
pub struct ManagerResources {
    chunk_entities_manager: ChunkEntitiesManager,
}

fn init_manager(mut commands: Commands, game_resources: Res<GameResources>) {
    let voxel_mesher = Arc::new(NaiveMesher::new(
        game_resources.block_type_storage.clone(),
        game_resources.texture_dictionary.clone(),
    ));

    let chunk_provider = Box::new(GeneratedChunkProvider::new());

    let chunk_manager_resource = ChunkManagerResources::new(voxel_mesher, chunk_provider);

    /*
    let manager_resources = ManagerResources {
        chunk_entities_manager: ChunkEntitiesManager::new(chunk_object_rx),
    };

    commands.insert_resource(manager_resources);
 */
    commands.insert_resource(chunk_manager_resource);
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
/*
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
 */
}

#[derive(Resource, Debug, Clone, PartialEq)]
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
}

impl ChunkManagerResources {
    fn new(
        voxel_mesher: Arc<dyn ChunkMesher>,
        chunk_provider: Box<dyn ChunkProvider>,
    ) -> Self {
        let chunk_loader = ChunkLoader::new(chunk_provider);

        let inner_builder = Box::new(AsyncChunkBuilder::new(voxel_mesher));
        let chunk_builder = Box::new(builders::versioned_chunk_builder::VersionedChunkBuilder::<()>::new(inner_builder));

        Self {
            world: PhysicalWorld::default(),
            chunk_loader,
            chunk_builder,
        }
    }
}

fn chunk_streamer(mut chunk_pos_changed_ev: EventReader<ChunkPosChangedEvent>, resources: ChunkManagerResources, config: Config) {
    for ev in chunk_pos_changed_ev.read() {
        info!("chunk pos: {:?}", ev.chunk_pos);
        let player_pos = ev.chunk_pos;

        // Remove all chunks that are still empty.
        let empty_chunks_in_world: Vec<ChunkPos> =
            resources.world.get_chunks_with_state(ChunkState::Empty);

        for pos in empty_chunks_in_world {
            resources.world.remove_chunk(pos);
        }


        // Load missing chunks within the load distance.
        let generator = ChunkPosGenerator2D::new(player_pos, config.load_distance);
        generator.for_each(|| {
            self.load_chunk_if_is_empty(pos);
        });

        // Update all existing chunks in the world.
        let chunks_in_world: Vec<ChunkPos> = self.world.chunk_states.keys().copied().collect();

        for pos in chunks_in_world {
            self.update_chunk_state(pos);
        }
    }
}

fn 