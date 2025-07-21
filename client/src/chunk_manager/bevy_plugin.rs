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
    chunk_manager::{physical_world::PhysicalWorld, ChunkState},
    chunk_mesh_builder::{
        builders::{self, async_chunk_builder::AsyncChunkBuilder, ChunkBuilder, Versioned},
        meshers::{naive_mesher::NaiveMesher, ChunkMesher},
        ChunkMesh,
    },
    controller,
    orchestrator::ChunkPosChangedEvent,
};

use super::{bevy_event_manager::WorldChunkUpdateEvent, ChunkEntitiesManager};

pub struct ChunkManagerPlugin;
impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Config {
            load_distance: 10,
            render_distance: 9,
            dynamic_vertical_loading: false,
        })
        .insert_resource(PhysicalWorldResource::default())
        .insert_resource(ChunkManagerResource::default())
        .add_systems(OnEnter(AppStates::InGame), init_manager)
        .add_event::<WorldChunkUpdateEvent>()
        .add_event::<ChunkLoaded>()
        .add_event::<ChunkBuilt>()
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

    let loader_resource = ChunkLoaderResource::new(chunk_provider);
    let builder_resource = ChunkBuilderResource::new(voxel_mesher);

    commands.insert_resource(loader_resource);
    commands.insert_resource(builder_resource);

    /*
       let manager_resources = ManagerResources {
           chunk_entities_manager: ChunkEntitiesManager::new(chunk_object_rx),
       };

       commands.insert_resource(manager_resources);
    */
}

fn update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    game_resources: Res<GameResources>,
    // mut chunk_manager_resources: ResMut<ChunkManagerResources>,
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
pub struct ChunkManagerResource {
    controller_pos: ChunkPos,
}

impl Default for ChunkManagerResource {
    fn default() -> Self {
        Self { controller_pos: ChunkPos::new(0, 0, 0) }
    }
}

#[derive(Resource, Default)]
pub struct PhysicalWorldResource {
    world: PhysicalWorld,
}

#[derive(Resource)]
pub struct ChunkLoaderResource {
    chunk_loader: ChunkLoader,
}

impl ChunkLoaderResource {
    fn new(chunk_provider: Box<dyn ChunkProvider>) -> Self {
        let chunk_loader = ChunkLoader::new(chunk_provider);

        Self { chunk_loader }
    }
}

#[derive(Resource)]
pub struct ChunkBuilderResource {
    chunk_builder: Box<dyn VersionedChunkBuilder<()>>,
}

impl ChunkBuilderResource {
    fn new(voxel_mesher: Arc<dyn ChunkMesher>) -> Self {
        let inner_builder = Box::new(AsyncChunkBuilder::new(voxel_mesher));
        let chunk_builder = Box::new(
            builders::versioned_chunk_builder::VersionedChunkBuilder::<()>::new(inner_builder),
        );

        Self { chunk_builder }
    }
}

fn update_controller_pos(
    mut chunk_pos_changed_ev: EventReader<ChunkPosChangedEvent>,
    mut chunk_manager_resource: ResMut<ChunkManagerResource>,
) {
    if let Some(ev) = chunk_pos_changed_ev.read().last() {
        if chunk_manager_resource.controller_pos != ev.chunk_pos {
            chunk_manager_resource.controller_pos = ev.chunk_pos;
        }
    }
}

fn chunk_streamer(
    mut chunk_pos_changed_ev: EventReader<ChunkPosChangedEvent>,
    mut world: ResMut<PhysicalWorldResource>,
    config: Res<Config>,
) {
    let mut world = &mut world.world;

    for ev in chunk_pos_changed_ev.read() {
        info!("chunk pos: {:?}", ev.chunk_pos);
        let player_pos = ev.chunk_pos;

        // Remove all chunks that are still empty.
        let empty_chunks_in_world: Vec<ChunkPos> = world.get_chunks_with_state(ChunkState::Empty);

        for pos in empty_chunks_in_world {
            world.remove_chunk(pos);
        }

        // Load missing chunks within the load distance.
        let generator = ChunkPosGenerator2D::new(player_pos, config.load_distance);
        generator.for_each(|pos| {
            // TODO
            // self.load_chunk_if_is_empty(pos);
        });

        // Update all existing chunks in the world.
        let chunks_in_world: Vec<ChunkPos> = world.chunk_states.keys().copied().collect();

        for pos in chunks_in_world {
            // TODO
            // self.update_chunk_state(pos);
        }
    }
}

/// Chunks loaded by `ChunkLoader`, not added to world.
#[derive(Event, Debug)]
struct ChunkLoaded {
    chunk_pos: ChunkPos,
    chunk: Chunk,
}

fn chunks_loader(
    mut chunk_loaded_ev: EventWriter<ChunkLoaded>,
    mut chunk_loader: ResMut<ChunkLoaderResource>,
    chunk_manager_resource: Res<ChunkManagerResource>,
) {
    let mut chunk_loader = &mut chunk_loader.chunk_loader;

    chunk_loader.update(chunk_manager_resource.controller_pos);

    for (chunk_pos, chunk) in chunk_loader.poll_loaded_chunks() {
        chunk_loaded_ev.send(ChunkLoaded { chunk_pos, chunk });
    }
}

/// Chunks built by `ChunkBuilder`, not added to world.
#[derive(Event, Debug)]
struct ChunkBuilt {
    chunk_pos: ChunkPos,
    chunk_mesh: ChunkMesh,
}

fn chunks_builder(
    mut chunk_built_ev: EventWriter<ChunkBuilt>,
    mut chunk_builder: ResMut<ChunkBuilderResource>,
    chunk_manager_resource: Res<ChunkManagerResource>,
) {
    let mut chunk_builder = &mut chunk_builder.chunk_builder;

    chunk_builder.update(chunk_manager_resource.controller_pos);

    for (chunk_pos, (chunk_mesh, _)) in chunk_builder.poll_completed() {
        chunk_built_ev.send(ChunkBuilt {
            chunk_pos,
            chunk_mesh,
        });
    }
}

