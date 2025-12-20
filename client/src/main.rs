use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::WHITE,
    ecs::system::command::unregister_system,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    reflect::TypeData,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        *,
    },
};
use bevy_asset_loader::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_common_assets::yaml::YamlAssetPlugin;

use bevy_render::VoxelRenderPlugin;
use bevy_resources::{MaterialsDictAsset, MaterialsDictAssetLoader};
use bevy_types::{AppStates, GameResources};
use cgmath::dot;
use chunk_mesh_builder::*;
use controller::ControllerPlugin;
use shared::{
    entities::{init_block_names, name_to_block_id, BlockID, BlockPos, BlockTypeStorage},
    physics::RaycastResult,
};

use chunk_manager::{ChunkManagerPlugin, ChunkManagerResources};

use crate::{
    bevy_render::{ColoredCubeMaterial, TexturedCubeMaterial},
    bevy_resources::{
        BevyBlockTypeStorageResource, MaterialHandle, MaterialStorage, MaterialsDictionary,
        RenderDescDictAsset, RenderDescDictAssetLoader, RenderDescDictionary, TextureAsset,
        TextureDictAsset, TextureDictAssetLoader, TextureDictionary, TextureIndexDictionary,
    },
    controller::ActionType,
    gui::GUIPlugin,
    orchestrator::{utils::raycast_from_controller, OrchestratorPlugin},
};

mod bevy_render;
mod bevy_resources;
mod bevy_types;
mod chunk_manager;
mod chunk_mesh_builder;
mod controller;
mod gui;
mod orchestrator;
mod voxel_edits;

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
                })
                .set(bevy::log::LogPlugin {
                    // level: bevy::log::Level::TRACE,
                    ..default()
                }),
            WireframePlugin::default(),
            JsonAssetPlugin::<BevyBlockTypeStorageResource>::new(&["blocks.json"]),
            ControllerPlugin,
            VoxelRenderPlugin,
            ChunkManagerPlugin,
            OrchestratorPlugin,
            GUIPlugin,
        ))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: WHITE.into(),
        })
        .init_asset_loader::<RenderDescDictAssetLoader>()
        .init_asset::<RenderDescDictAsset>()
        .init_asset_loader::<MaterialsDictAssetLoader>()
        .init_asset::<MaterialsDictAsset>()
        .init_asset_loader::<TextureDictAssetLoader>()
        .init_asset::<TextureDictAsset>()
        .init_asset::<BevyBlockTypeStorageResource>()
        .init_state::<AppStates>()
        .add_loading_state(
            LoadingState::new(AppStates::Loading)
                .continue_to_state(AppStates::InGame)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "texture_array.assets.ron",
                )
                .load_collection::<VoxelAssets>(),
        )
        .add_systems(OnExit(AppStates::Loading), create_resources)
        .add_systems(OnExit(AppStates::Loading), init_level)
        .add_systems(Update, update.run_if(in_state(AppStates::InGame)))
        .run();
}

#[derive(AssetCollection, Resource)]
struct VoxelAssets {
    #[asset(path = "global.render_desc.json")]
    render_desc_storage_res: Handle<RenderDescDictAsset>,
    #[asset(key = "opaque")]
    opaque_texture: Handle<Image>,
    #[asset(path = "global.textures.yaml")]
    texture_dict_asset: Handle<TextureDictAsset>,
    #[asset(path = "global.blocks.json")]
    server_blocks: Handle<BevyBlockTypeStorageResource>,
    #[asset(path = "textures/palette.png")]
    color_palette: Handle<Image>,
    #[asset(path = "global.materials.json")]
    materials_dict_asset: Handle<MaterialsDictAsset>,
}

fn create_resources(
    mut commands: Commands,
    mut textured_materials: ResMut<Assets<TexturedCubeMaterial>>,
    mut colored_materials: ResMut<Assets<ColoredCubeMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    mut texture_dict_asset: ResMut<Assets<TextureDictAsset>>,
    mut render_desc_dict_asset: ResMut<Assets<RenderDescDictAsset>>,
    mut materials_dict_asset: ResMut<Assets<MaterialsDictAsset>>,
    server_block_type_assets: Res<Assets<BevyBlockTypeStorageResource>>,
    voxel_assets: Res<VoxelAssets>,
    asset_server: Res<AssetServer>,
) {
    let render_desc_dict_asset: RenderDescDictAsset = render_desc_dict_asset
        .remove(&voxel_assets.render_desc_storage_res)
        .unwrap();
    let render_desc_dict: Arc<RenderDescDictionary> = Arc::new(render_desc_dict_asset.0);

    let texture_dictionary_asset: TextureDictAsset = texture_dict_asset
        .remove(&voxel_assets.texture_dict_asset)
        .unwrap();
    let texture_dictionary: Arc<TextureDictionary> = Arc::new(texture_dictionary_asset.into());

    dbg!(&texture_dictionary);

    let server_block_type_storage_asset = server_block_type_assets
        .get(&voxel_assets.server_blocks)
        .expect("Failed to get server_block_type_storage asset")
        .to_owned();
    let server_block_type_storage: Arc<BlockTypeStorage> =
        Arc::new(server_block_type_storage_asset.clone().into());

    let materials_dict_asset: MaterialsDictAsset = materials_dict_asset
        .remove(&voxel_assets.materials_dict_asset)
        .unwrap();
    let materials_dict: Arc<MaterialsDictionary> = Arc::new(materials_dict_asset.0);

    dbg!(&materials_dict);

    let color_palette = textures
        .get(&voxel_assets.color_palette)
        .expect("Failed to load color palette.")
        .to_owned();
    let color_palette = create_1d_color_palette(color_palette);
    let color_palette_handle = textures.add(color_palette);

    let material_storage = create_material_storage(
        &mut textured_materials,
        &mut colored_materials,
        &materials_dict,
        voxel_assets.opaque_texture.clone(),
        color_palette_handle.clone(),
    );

    let result = texture_dictionary.compile(asset_server);
    dbg!(result);

    // TODO: make this flexible.
    let texture_index_dictionary: &TextureIndexDictionary =
        match texture_dictionary.get(&"opaque".to_string()).unwrap() {
            TextureAsset::TextureArray { data } => &data.textures,
            TextureAsset::Palette { data } => unimplemented!(),
        };
    let texture_index_dictionary = Arc::new(texture_index_dictionary.clone());

    commands.insert_resource(GameResources {
        render_desc_dict,
        server_block_type_storage,
        texture_index_dictionary,
        material_storage,
    });

    init_block_names(server_block_type_storage_asset.into());
}

fn create_1d_color_palette(color_palette_2d: Image) -> Image {
    assert_eq!(color_palette_2d.width(), 256);
    assert_eq!(color_palette_2d.height(), 1);

    let extend = Extent3d {
        width: 256,
        height: 1,
        ..default()
    };

    let data = color_palette_2d.data.unwrap();
    let format = color_palette_2d.texture_descriptor.format;

    Image::new(
        extend,
        TextureDimension::D1,
        data,
        format,
        RenderAssetUsages::default(),
    )
}

fn create_material_storage(
    textured_materials: &mut ResMut<Assets<TexturedCubeMaterial>>,
    colored_materials: &mut ResMut<Assets<ColoredCubeMaterial>>,
    materials_dict: &Arc<MaterialsDictionary>,
    opaque_texture: Handle<Image>,
    color_palette: Handle<Image>,
) -> Arc<MaterialStorage> {
    let mut material_storage = MaterialStorage::new(Vec::default());

    let textured_mat = TexturedCubeMaterial {
        array_texture: opaque_texture,
    };
    let textured_mat_handle = textured_materials.add(textured_mat);

    let colored_mat = ColoredCubeMaterial { color_palette };
    let colored_mat_handle = colored_materials.add(colored_mat);

    material_storage.add(MaterialHandle::TexturedCube(textured_mat_handle));
    material_storage.add(MaterialHandle::ColoredCube(colored_mat_handle));

    Arc::new(material_storage)
}

fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
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
}

fn update(
    mut chunk_manager_resources: ResMut<ChunkManagerResources>,
    game_resources: ResMut<GameResources>,
    mut controller_ev: EventReader<controller::ActionEvent>,
) {
    for ev in controller_ev.read() {
        let world = &chunk_manager_resources.chunk_manager.get_world().world;
        let raycast_result = raycast_from_controller(
            ev.controller_pos,
            ev.controller_forward,
            world,
            &game_resources.server_block_type_storage,
        );

        if raycast_result.collide {
            let block_action: BlockAction = match ev.action_type {
                ActionType::LeftClick => destroy_block_action(raycast_result),
                ActionType::RightClick => place_block_action(raycast_result),
            };

            if block_action.feasible {
                voxel_edits::set_block_and_update_chunk(
                    &mut chunk_manager_resources.chunk_manager,
                    block_action.pos,
                    block_action.new_block,
                );
            }
        } else {
            println!("Raycast don't collide.");
        }
    }
}

struct BlockAction {
    feasible: bool,
    pos: BlockPos,
    new_block: BlockID,
}

fn destroy_block_action(raycast_result: RaycastResult) -> BlockAction {
    BlockAction {
        feasible: true,
        pos: raycast_result.hitpoint.pos,
        new_block: name_to_block_id("air"),
    }
}

fn place_block_action(raycast_result: RaycastResult) -> BlockAction {
    let previous_block_id = raycast_result.step_before_hitpoint.block_id;

    BlockAction {
        feasible: previous_block_id == name_to_block_id("air"),
        pos: raycast_result.step_before_hitpoint.pos,
        new_block: name_to_block_id("wood"),
    }
}
