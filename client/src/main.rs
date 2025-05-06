use std::rc::Rc;

use bevy::prelude::*;
use bevy::render::{
    settings::{RenderCreation, WgpuSettings},
    RenderPlugin,
};
use bevy::{
    color::palettes::css::*,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    render::settings::WgpuFeatures,
};
use bevy_render::{BevyChunkEntity, BevyChunkMesh};
use bevy_resources::block_types::{MeshBlockTypeStorageLoader, MeshBlockTypeStorageResource};
use bevy_asset_loader::prelude::*;
use chunk_builder::*;
use controller::ControllerPlugin;
use shared::{
    chunk_loader::*,
    entities::{world, Chunk, ChunkPos},
};

mod bevy_render;
mod bevy_resources;
mod chunk_builder;
mod controller;

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
            ControllerPlugin,
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
                .load_collection::<VoxelAssets>(),
        )
        .add_systems(OnEnter(AppStates::InGame), init_level)
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
}

fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    voxel_assets: Res<VoxelAssets>,
    assets: Res<Assets<MeshBlockTypeStorageResource>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(5.0)))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)),
        GlobalTransform::default(),
    ));

    commands.spawn((
        DirectionalLight { ..default() },
        Transform::from_xyz(11.0, 20.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
    ));

    let mut chunk_loader = ChunkLoader::default();
    let mut world = world::World::new();
    let pos = ChunkPos::new(0, 0, 0);
    world.add_chunk(pos, chunk_loader.load_chunk(pos));
    let block_type_storage = assets.get(&voxel_assets.block_type_storage).unwrap();
    
    if let Some(chunk) = world.get_chunk(pos) {
        let mesh: BevyChunkMesh = build_chunk(chunk, Rc::new((block_type_storage.to_owned()).into()));
        let chunk_entity = BevyChunkEntity::new(mesh, commands, meshes, materials);
    }
}

fn build_chunk(chunk: &Chunk, block_type_storage: Rc<BlockTypeStorage>) -> BevyChunkMesh {
    let texture_dictionary = Rc::new(TextureDictionary::new());
    let mut voxel_mesher = VoxelMesher::new(
        chunk.get_block_storage().clone(),
        block_type_storage,
        texture_dictionary,
    );

    let chunk_mesh = voxel_mesher.create_mesh().clone();

    if let Some(err) = voxel_mesher.get_last_err() {
        eprintln!("{}", err);
    }

    BevyChunkMesh::from(chunk_mesh)
}
