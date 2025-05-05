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
use chunk_builder::*;
use controller::ControllerPlugin;
use shared::{
    chunk_loader::*,
    entities::{world, BlockSide, BlockType, Chunk, ChunkPos},
};

mod bevy_render;
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
        .add_systems(Startup, init_level)
        .run();
}

fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    if let Some(chunk) = world.get_chunk(pos) {
        let mesh: BevyChunkMesh = build_chunk(chunk);
        let chunk_entity = BevyChunkEntity::new(mesh, commands, meshes, materials);
    }
}

fn build_chunk(chunk: &Chunk) -> BevyChunkMesh {
    let air = BlockType::new("air", false);
    let air = MeshBlockTypeBuilder::new(air).translucent(true).build();
    let dirt = BlockType::new("dirt", true);
    let dirt = MeshBlockTypeBuilder::new(dirt)
        .visible(true)
        .texture(BlockSide::Front, "dirt")
        .build();

    let mut block_type_storage = BlockTypeStorage::new();
    block_type_storage.set_block_type(0, air);
    block_type_storage.set_block_type(1, dirt);

    let texture_dictionary = Rc::new(TextureDictionary::new());
    let mut voxel_mesher = VoxelMesher::new(
        chunk.get_block_storage().clone(),
        Rc::new(block_type_storage),
        texture_dictionary,
    );
    let chunk_mesh = voxel_mesher.create_mesh();

    BevyChunkMesh::from(chunk_mesh.clone())
}
