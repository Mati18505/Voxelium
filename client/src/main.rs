use std::{collections::HashMap, hash::Hash, rc::Rc, sync::{Arc, Mutex}};

use bevy::{
    color::palettes::css::WHITE,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        *,
    }, tasks::{futures_lite::{self, future}, AsyncComputeTaskPool, Task},
};
use bevy_asset_loader::prelude::*;
use bevy_common_assets::yaml::YamlAssetPlugin;

use bevy_render::{BevyChunkEntity, BevyChunkMesh, VoxelMaterial, VoxelRenderPlugin};
use bevy_resources::{MeshBlockTypeStorageLoader, MeshBlockTypeStorageResource, TextureConfig};
use chunk_builder::*;
use controller::ControllerPlugin;

use shared::{
    chunk_loader::*,
    entities::{world, BlockPos, Chunk, ChunkPos},
};

use crate::chunk_manager::{physical_world::{self, PhysicalWorld}, ChunkManager};

mod bevy_render;
mod bevy_resources;
mod chunk_builder;
mod controller;
mod chunk_manager;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                }),
            WireframePlugin::default(),
            YamlAssetPlugin::<TextureConfig>::new(&["config.yaml"]),
            ControllerPlugin,
            VoxelRenderPlugin,
        ))
        .insert_resource(WireframeConfig {
            global: true,
            default_color: WHITE.into(),
        })
        .init_asset_loader::<MeshBlockTypeStorageLoader>()
        .init_asset::<MeshBlockTypeStorageResource>()
        .init_state::<AppStates>()
        .add_loading_state(
            LoadingState::new(AppStates::Loading)
                .continue_to_state(AppStates::InGame)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "texture_array.assets.ron",
                )
                .load_collection::<VoxelAssets>(),
        )
        .add_systems(OnEnter(AppStates::InGame), init_level)
        .add_systems(Update, update.run_if(in_state(AppStates::InGame)))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AppStates {
    #[default]
    Loading,
    InGame,
}

#[derive(AssetCollection, Resource)]
struct VoxelAssets {
    #[asset(path = "global.blocks.json")]
    block_type_storage: Handle<MeshBlockTypeStorageResource>,
    #[asset(key = "opaque")]
    opaque_texture: Handle<Image>,
    #[asset(path = "textures.config.yaml")]
    texture_config: Handle<TextureConfig>,
}

#[derive(Resource)]
struct GameResources {
    chunk_manager: ChunkManager,
    chunk_entities_manager: Arc<Mutex<ChunkEntitiesManager>>,
}

fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
    block_type_assets: Res<Assets<MeshBlockTypeStorageResource>>,
    textures_assets: Res<Assets<TextureConfig>>,
    voxel_assets: Res<VoxelAssets>,
    mut voxel_materials: ResMut<Assets<VoxelMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(5.0)))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)),
        GlobalTransform::default(),
    ));

    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 100.0;

    commands.spawn((
        DirectionalLight { ..default() },
        Transform::from_xyz(11.0, 20.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));

    let block_type_storage = block_type_assets
        .get(&voxel_assets.block_type_storage)
        .unwrap()
        .to_owned();
    let block_type_storage: Arc<BlockTypeStorage> = Arc::new(block_type_storage.into());

    let texture_dictionary: TextureConfig = textures_assets
        .get(&voxel_assets.texture_config)
        .unwrap()
        .to_owned();
    let texture_dictionary: Arc<TextureDictionary> = Arc::new(texture_dictionary.into());

    let chunk_loader = ChunkLoader::default();
    let chunk_builder = Box::new(ChunkBuilder::new(block_type_storage, texture_dictionary));
    let config = chunk_manager::Config::new(5, 4);

    let chunk_entities_manager = Arc::new(Mutex::new(ChunkEntitiesManager::default()));
    let mut chunk_manager = ChunkManager::new(chunk_loader, chunk_builder, config);
    chunk_manager.set_chunk_object_callback(chunk_entities_manager.clone());

    let game_resources = GameResources {
        chunk_manager,
        chunk_entities_manager,
    };
    commands.insert_resource(game_resources);
}

fn update(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
    voxel_assets: Res<VoxelAssets>,
    block_type_assets: Res<Assets<MeshBlockTypeStorageResource>>,
    textures_assets: Res<Assets<TextureConfig>>,
    mut voxel_materials: ResMut<Assets<VoxelMaterial>>,
    mut game_resources: ResMut<GameResources>,
    mut controller_events: EventReader<controller::PositionChangeEvent>,
) {
    for e in controller_events.read() {
        let new_pos = e.new_pos;
        // Convert bevy direction to our direction
        let new_block_pos = BlockPos::new(new_pos.x as isize, -new_pos.z as isize, new_pos.y as isize);
        let new_chunk_pos = ChunkPos::from(new_block_pos);

        let need_redraw = game_resources.chunk_manager.update_controller_pos(new_chunk_pos);

        if need_redraw {
            println!("need redraw");

            let physical_world = game_resources.chunk_manager.get_world().clone();

            println!("Chunk meshes: {}", physical_world.chunk_meshes.len());
            println!("Chunk states: {}", physical_world.chunk_states.len());
        }
    }

    game_resources.chunk_manager.check_builded_chunks();

    if let Ok(mut manager) = game_resources.chunk_entities_manager.lock() {
        manager.process_pending(&mut commands, &mut meshes, &voxel_assets, &mut voxel_materials);
    }
}

#[derive(Resource)]
struct ChunkBuildTask(Task<ChunkMesh>);

struct ChunkBuilder {
    block_type_storage: Arc<BlockTypeStorage>,
    texture_dictionary: Arc<TextureDictionary>,
    tasks: HashMap<ChunkPos, ChunkBuildTask>,
}

impl ChunkBuilder {
    fn new(
        block_type_storage: Arc<BlockTypeStorage>,
        texture_dictionary: Arc<TextureDictionary>,
    ) -> ChunkBuilder {
        ChunkBuilder {
            block_type_storage,
            texture_dictionary,
            tasks: HashMap::default(),
        }
    }
}

impl chunk_manager::ChunkBuilder for ChunkBuilder {
    fn build_chunk(
        &mut self,
        chunk_pos: ChunkPos,
        chunk: &Chunk,
    ) {
        assert!(!self.tasks.contains_key(&chunk_pos), "Can't build a fragment at the same position a second time.");

        // TODO
        // Chunk Grouping (for each thread job give multiple chunks).
        // What if chunk needs to be redrawn before being build? Assert will crash the application.

        let mut voxel_mesher = VoxelMesher::new(
            chunk.get_block_storage().clone(),
            self.block_type_storage.clone(),
            self.texture_dictionary.clone(),
        );

        let pool = AsyncComputeTaskPool::get();
        let task = pool.spawn(async move {
            let chunk_mesh = voxel_mesher.create_mesh().clone();

            if let Some(err) = voxel_mesher.get_last_err() {
                eprintln!("{}", err);
            }

            chunk_mesh
        });

        self.tasks.insert(chunk_pos, ChunkBuildTask(task));
    }

    fn get_builded_chunks(&mut self) -> HashMap<ChunkPos, ChunkMesh> {
        let mut completed: HashMap<ChunkPos, ChunkMesh> = HashMap::default();

        for (chunk_pos, build_task) in self.tasks.iter_mut() {
            if let Some(chunk_mesh) = future::block_on(future::poll_once(&mut build_task.0)) {
                completed.insert(*chunk_pos, chunk_mesh);
            }
        }

        for chunk_pos in completed.keys() {
            self.tasks.remove(chunk_pos);
        }

        completed
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct ChunkEntitiesManager {
    pending_to_create: HashMap<ChunkPos, ChunkMesh>,
    pending_to_remove: Vec<ChunkPos>,
    chunk_entities: HashMap<ChunkPos, BevyChunkEntity>,
}

impl chunk_manager::ChunkObjectCallback for ChunkEntitiesManager {
    fn chunk_object_created(&mut self, chunk_pos: ChunkPos, chunk_mesh: &ChunkMesh) {
        self.pending_to_create.insert(chunk_pos, chunk_mesh.clone());
    }
    fn chunk_object_removed(&mut self, chunk_pos: ChunkPos) {
        self.pending_to_remove.push(chunk_pos);
    }
}

impl ChunkEntitiesManager {
    pub fn process_pending(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        voxel_assets: &Res<VoxelAssets>,
        voxel_materials: &mut ResMut<Assets<VoxelMaterial>>,
    ) {
        for (pos, mesh) in std::mem::take(&mut self.pending_to_create) {
            self.create_chunk_entity(pos, mesh, commands, meshes, voxel_assets, voxel_materials);
        }

        for pos in std::mem::take(&mut self.pending_to_remove) {
            self.remove_chunk_entity(&pos, commands);
        }

        assert!(self.pending_to_create.len() == 0);
        assert!(self.pending_to_remove.len() == 0);
    }

    fn create_chunk_entity(
        &mut self, 
        pos: ChunkPos,
        mesh: ChunkMesh,
        mut commands: &mut Commands,
        mut meshes: &mut ResMut<Assets<Mesh>>,
        voxel_assets: &Res<VoxelAssets>,
        mut voxel_materials: &mut ResMut<Assets<VoxelMaterial>>,
    ) {
        let mut mesh = BevyChunkMesh::from(mesh);
        mesh.apply_transform(Transform::from_xyz(pos.x as f32, pos.y as f32, pos.z as f32));

        let chunk_entity = BevyChunkEntity::new(
            mesh,
            &mut commands,
            &mut meshes,
            &mut voxel_materials,
            voxel_assets.opaque_texture.clone(),
        );

        self.chunk_entities.insert(pos, chunk_entity);
    }

    fn remove_chunk_entity(
        &mut self,
        pos: &ChunkPos,
        mut commands: &mut Commands,
    ) {
        if let Some(entity) = self.chunk_entities.get(pos) {
            entity.cleanup(&mut commands);
            self.chunk_entities.remove(pos);
        }
    }
}