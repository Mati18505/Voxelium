use bevy::{
    color::palettes::css::WHITE,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        *,
    },
};

use std::ops::Deref;

use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_render::VoxelRenderPlugin;
use bevy_types::{AppStates, GameResources};
use controller::ControllerPlugin;
use shared::{
    entities::{BlockID, BlockPos, BlockRegistry},
    physics::RaycastResult,
};

use chunk_manager::ChunkManagerPlugin;

use crate::{
    assets::AssetsPlugin,
    bevy_resources::{BlockNameToId, ResourcesPlugin},
    chunk_manager::{ChunkStorage, VoxelEdit},
    controller::ActionType,
    diagnostics::{DiagnosticsConfig, DiagnosticsPlugin},
    gui::GUIPlugin,
    orchestrator::{utils::raycast_from_controller, OrchestratorPlugin},
};

mod assets;
mod bevy_render;
mod bevy_resources;
mod bevy_types;
mod chunk_manager;
mod chunk_mesh_builder;
mod controller;
mod diagnostics;
mod gui;
mod orchestrator;
mod voxel_render_core;

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
            AssetsPlugin,
            ControllerPlugin,
            VoxelRenderPlugin,
            ChunkManagerPlugin,
            OrchestratorPlugin,
            GUIPlugin,
            ResourcesPlugin,
            InfiniteGridPlugin,
            DiagnosticsPlugin::new(DiagnosticsConfig {}),
        ))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: WHITE.into(),
        })
        .init_state::<AppStates>()
        .add_systems(OnExit(AppStates::Compile), init_level)
        .add_observer(on_action_event)
        .run();
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
    commands.spawn(InfiniteGridBundle::default());
}

fn on_action_event(
    action: On<controller::ActionEvent>,
    mut voxel_edits: MessageWriter<VoxelEdit>,
    state: Res<State<AppStates>>,
    chunks: Option<ChunkStorage>,
    game_resources: Option<Res<GameResources>>,
    registry: Res<BlockNameToId>,
) {
    if !matches!(state.get(), AppStates::InGame) {
        return;
    }
    let (Some(game_resources), Some(chunks)) = (game_resources, chunks) else {
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
            ActionType::LeftClick => destroy_block_action(raycast_result, registry.deref()),
            ActionType::RightClick => place_block_action(raycast_result, registry.deref()),
        };

        if block_action.feasible {
            voxel_edits.write(VoxelEdit(block_action.pos, block_action.new_block));
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

fn destroy_block_action(raycast_result: RaycastResult, registry: &impl BlockRegistry) -> BlockAction {
    BlockAction {
        feasible: true,
        pos: raycast_result.hitpoint.pos,
        new_block: registry.name_to_block_id("air"),
    }
}

fn place_block_action(raycast_result: RaycastResult, registry: &impl BlockRegistry) -> BlockAction {
    let previous_block_id = raycast_result.step_before_hitpoint.block_id;

    BlockAction {
        feasible: previous_block_id == registry.name_to_block_id("air"),
        pos: raycast_result.step_before_hitpoint.pos,
        new_block: registry.name_to_block_id("wood"),
    }
}
