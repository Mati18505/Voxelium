use std::sync::Arc;

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

use bevy_render::VoxelRenderPlugin;
use bevy_resources::{MeshBlockTypeStorageLoader, MeshBlockTypeStorageResource, TextureConfig};
use chunk_builder::*;
use controller::ControllerPlugin;
use bevy_types::{AppStates, GameResources};
use shared::chunk_loader::ChunkLoader;

use chunk_manager::{ChunkManagerPlugin, ChunkManagerResources};

mod bevy_render;
mod bevy_resources;
mod chunk_builder;
mod controller;
mod chunk_manager;
mod bevy_types;

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
            ChunkManagerPlugin,
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
        .add_systems(OnExit(AppStates::Loading), init_level)
        .add_systems(Update, update.run_if(in_state(AppStates::InGame)))
        .run();
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
    block_type_assets: Res<Assets<MeshBlockTypeStorageResource>>,
    textures_assets: Res<Assets<TextureConfig>>,
    voxel_assets: Res<VoxelAssets>,
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

    commands.insert_resource(GameResources{
        block_type_storage,
        texture_dictionary,
        opaque_texture: voxel_assets.opaque_texture.clone(),
    });
}

fn update() {

}