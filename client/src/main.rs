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
use bevy_common_assets::json::JsonAssetPlugin;

use bevy_render::VoxelRenderPlugin;
use bevy_resources::{MaterialsDictAsset, MaterialsDictAssetLoader};
use bevy_types::{AppStates, GameResources};
use controller::ControllerPlugin;
use shared::{
    entities::{name_to_block_id, BlockID, BlockPos},
    physics::RaycastResult,
};

use chunk_manager::{ChunkManagerPlugin, ChunkManagerResources};

use crate::{
    bevy_resources::{
        BevyBlockTypeStorageAsset, RenderDescDictAsset, RenderDescDictAssetLoader, ResourcesPlugin,
        TextureDictAsset, TextureDictAssetLoader,
    }, chunk_manager::ChunkStorage, controller::ActionType, gui::GUIPlugin, orchestrator::{utils::raycast_from_controller, OrchestratorPlugin}
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
            JsonAssetPlugin::<BevyBlockTypeStorageAsset>::new(&["blocks.json"]),
            ControllerPlugin,
            VoxelRenderPlugin,
            ChunkManagerPlugin,
            OrchestratorPlugin,
            GUIPlugin,
            ResourcesPlugin,
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
        .init_asset::<BevyBlockTypeStorageAsset>()
        .init_state::<AppStates>()
        .add_loading_state(
            LoadingState::new(AppStates::Loading)
                .continue_to_state(AppStates::Compile)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "texture_array.assets.ron",
                )
                .load_collection::<VoxelAssets>(),
        )
        .add_systems(OnExit(AppStates::Compile), init_level)
        .add_observer(on_action_event)
        .run();
}

#[derive(AssetCollection, Resource)]
struct VoxelAssets {
    #[asset(path = "global.render_desc.json")]
    render_desc_storage_res: Handle<RenderDescDictAsset>,
    #[asset(path = "global.textures.yaml")]
    texture_dict_asset: Handle<TextureDictAsset>,
    #[asset(path = "global.blocks.json")]
    server_blocks: Handle<BevyBlockTypeStorageAsset>,
    #[asset(path = "global.materials.json")]
    materials_dict_asset: Handle<MaterialsDictAsset>,
}

fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<bevy::light::GlobalAmbientLight>,
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

fn on_action_event(
    action: On<controller::ActionEvent>,
    state: Res<State<AppStates>>,
    chunks: Option<ResMut<ChunkStorage>>,
    game_resources: Option<Res<GameResources>>,
) {
    if !matches!(state.get(), AppStates::InGame) {
        return;
    }
    let (Some(game_resources), Some(mut chunks)) =
        (game_resources, chunks)
    else {
        return;
    };

    let raycast_result = raycast_from_controller(
        action.controller_pos,
        action.controller_forward,
        &chunks,
        &game_resources.server_block_type_storage,
    );

    if raycast_result.collide {
        let block_action: BlockAction = match action.action_type {
            ActionType::LeftClick => destroy_block_action(raycast_result),
            ActionType::RightClick => place_block_action(raycast_result),
        };

        if block_action.feasible {
            voxel_edits::set_block_and_update_chunk(
                &mut chunks.0,
                block_action.pos,
                block_action.new_block,
            );
        }
    } else {
        println!("Raycast don't collide.");
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
