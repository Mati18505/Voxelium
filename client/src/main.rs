use std::rc::Rc;

use bevy::{
    color::palettes::css::WHITE,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        *,
    },
};
use bevy_asset_loader::prelude::*;
use bevy_common_assets::yaml::YamlAssetPlugin;

use bevy_render::{BevyChunkEntity, BevyChunkMesh, VoxelMaterial, VoxelRenderPlugin};
use bevy_resources::{MeshBlockTypeStorageLoader, MeshBlockTypeStorageResource, TextureConfig};
use chunk_builder::*;
use controller::ControllerPlugin;

use shared::{
    chunk_loader::*,
    entities::{world, Chunk, ChunkPos},
};

use crate::chunk_manager::ChunkManager;

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

fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
    voxel_assets: Res<VoxelAssets>,
    block_type_assets: Res<Assets<MeshBlockTypeStorageResource>>,
    textures_assets: Res<Assets<TextureConfig>>,
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
    let block_type_storage: Rc<BlockTypeStorage> = Rc::new(block_type_storage.into());

    let texture_dictionary: TextureConfig = textures_assets
        .get(&voxel_assets.texture_config)
        .unwrap()
        .to_owned();
    let texture_dictionary: Rc<TextureDictionary> = Rc::new(texture_dictionary.into());

    let mut chunk_loader = ChunkLoader::default();
    let mut chunk_builder = Box::new(ChunkBuilder {
        block_type_storage,
        texture_dictionary,
    });
    let config = chunk_manager::Config {
        load_distance: 6,
        render_distance: 4,
    };

    let mut chunk_manager = ChunkManager::new(chunk_loader, chunk_builder, config);
    chunk_manager.update(ChunkPos::new(0,0,0));

    for (pos, mesh) in chunk_manager.get_world().chunk_meshes.iter() {
        let mesh = BevyChunkMesh::from(mesh.clone());
        let chunk_entity = BevyChunkEntity::new(
            mesh,
            &mut commands,
            &mut meshes,
            &mut voxel_materials,
            voxel_assets.opaque_texture.clone(),
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ChunkBuilder {
    block_type_storage: Rc<BlockTypeStorage>,
    texture_dictionary: Rc<TextureDictionary>,
}

impl chunk_manager::ChunkBuilder for ChunkBuilder {
    fn build_chunk(
        &self,
        chunk: &Chunk,
    ) -> ChunkMesh {
        let mut voxel_mesher = VoxelMesher::new(
            chunk.get_block_storage().clone(),
            self.block_type_storage.clone(),
            self.texture_dictionary.clone(),
        );

        let chunk_mesh = voxel_mesher.create_mesh().clone();

        if let Some(err) = voxel_mesher.get_last_err() {
            eprintln!("{}", err);
        }

        chunk_mesh
    }
}
