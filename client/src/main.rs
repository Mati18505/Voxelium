use std::rc::Rc;

use bevy::{
    app::{App, Startup},
    prelude::*,
    DefaultPlugins,
};
use controller::ControllerPlugin;
use chunk_builder::*;
use shared::entities::{world, BlockSide, BlockType, Chunk, ChunkPos};
use shared::chunk_loader::*;

mod chunk_builder;
mod controller;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            ControllerPlugin,
        ))
        .add_systems(Startup, init_level)
        .run();
}


fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PointLight { ..default() },
        Transform::from_xyz(10.0, 20.0, 4.0),
        GlobalTransform::default(),
    ));

    let mut chunk_loader = ChunkLoader::default();
    let mut world = world::World::new();
    let pos = ChunkPos::new(0, 0, 0);

    world.add_chunk(pos, chunk_loader.load_chunk(pos));

    match world.get_chunk(pos) {
        Some(chunk) => build_chunk(chunk),
        None => (),
    }
}

fn build_chunk(chunk: &Chunk) {
    let air = BlockType::new("air", false);
    let air = MeshBlockTypeBuilder::new(air).build();
    let dirt = BlockType::new("dirt", true);
    let dirt = MeshBlockTypeBuilder::new(dirt).visible(true).texture(BlockSide::Front, "dirt").build();

    let mut block_type_storage = BlockTypeStorage::new();
    block_type_storage.set_block_type(0, air);
    block_type_storage.set_block_type(1, dirt);

    let texture_dictionary = Rc::new(TextureDictionary::new());
    VoxelMesher::new(chunk.get_block_storage().clone(), Rc::new(block_type_storage), texture_dictionary);
}