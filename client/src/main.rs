use std::{fs, sync::Arc};

use bevy::{
    color::palettes::css::WHITE, pbr::wireframe::{WireframeConfig, WireframePlugin}, prelude::*, render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        *,
    }
};
use bevy_asset_loader::prelude::*;
use bevy_common_assets::yaml::YamlAssetPlugin;
use bevy_common_assets::json::JsonAssetPlugin;

use bevy_render::VoxelRenderPlugin;
use bevy_resources::{MeshBlockTypeStorageLoader, MeshBlockTypeStorageResource, TextureConfig};
use cgmath::Vector3;
use chunk_builder::*;
use controller::ControllerPlugin;
use bevy_types::{AppStates, GameResources};
use shared::{entities::{init_block_names, name_to_block_id, BlockID, BlockInChunkPos, BlockPos, BlockTypeStorage, Chunk, ChunkPos, ChunkRepository}, physics::{raycast, RaycastConfig, RaycastResult}, resources::BlockTypeStorageResource};

use chunk_manager::{ChunkManagerPlugin, ChunkManagerResources};

use crate::{bevy_resources::BevyBlockTypeStorageResource, chunk_manager::{ChunkManager, WorldChunkUpdateEvent}, controller::ActionType};

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
            JsonAssetPlugin::<BevyBlockTypeStorageResource>::new(&["server_blocks.json"]),
            ControllerPlugin,
            VoxelRenderPlugin,
            ChunkManagerPlugin,
        ))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: WHITE.into(),
        })
        .init_asset_loader::<MeshBlockTypeStorageLoader>()
        .init_asset::<MeshBlockTypeStorageResource>()
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
        .add_systems(OnExit(AppStates::Loading), init_level)
        .add_systems(Update, (update, update_gizmo).run_if(in_state(AppStates::InGame)))
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
    #[asset(path = "global.server_blocks.json")]
    server_blocks: Handle<BevyBlockTypeStorageResource>,
}

fn init_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
    block_type_assets: Res<Assets<MeshBlockTypeStorageResource>>,
    server_block_type_assets: Res<Assets<BevyBlockTypeStorageResource>>,
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
    let block_type_storage: Arc<MeshBlockTypeStorage> = Arc::new(block_type_storage.into());

    let texture_dictionary: TextureConfig = textures_assets
        .get(&voxel_assets.texture_config)
        .unwrap()
        .to_owned();
    let texture_dictionary: Arc<TextureDictionary> = Arc::new(texture_dictionary.into());

    let server_block_type_storage_asset = server_block_type_assets
        .get(&voxel_assets.server_blocks)
        .expect("Failed to get server_block_type_storage asset")
        .to_owned();
    let server_block_type_storage: Arc<BlockTypeStorage> = Arc::new(server_block_type_storage_asset.clone().into());

    commands.insert_resource(GameResources{
        block_type_storage,
        server_block_type_storage,
        texture_dictionary,
        opaque_texture: voxel_assets.opaque_texture.clone(),
    });

    commands.spawn((
        GizmoData::default(),
    ));

    init_block_names(server_block_type_storage_asset.into());
}

fn update(
    mut chunk_manager_resources: ResMut<ChunkManagerResources>,
    game_resources: ResMut<GameResources>,
    mut controller_ev: EventReader<controller::ActionEvent>,
) {
    for ev in controller_ev.read() {
        let world = &chunk_manager_resources.chunk_manager.get_world().world;
        let raycast_result = raycast_from_controller(ev.controller_pos, ev.controller_forward, world, &game_resources.server_block_type_storage);

        if raycast_result.collide {
            let block_action: BlockAction = match ev.action_type {
                ActionType::LeftClick => destroy_block_action(raycast_result),
                ActionType::RightClick => place_block_action(raycast_result),
            };

            if block_action.feasible {
                set_block_and_update_chunk(&mut chunk_manager_resources.chunk_manager, block_action.pos, block_action.new_block);
            }
        } else {
            println!("Raycast don't collide.");
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct GizmoData {
    visible: bool,
    block_pos: BlockPos,
    last_controller_pos: Vec3,
    last_looking_dir: Vec3,
}

impl Default for GizmoData {
    fn default() -> Self {
        Self {
            visible: false,
            block_pos: BlockPos::new(0, 0, 0),
            last_controller_pos: Default::default(),
            last_looking_dir: Default::default() 
        }
    }
}

fn update_gizmo(
    mut gizmos: Gizmos,
    chunk_manager_resources: ResMut<ChunkManagerResources>,
    game_resources: ResMut<GameResources>,
    mut controller_position_change_ev: EventReader<controller::PositionChangeEvent>,
    mut controller_looking_dir_change_ev: EventReader<controller::LookingDirChangeEvent>,
    mut gizmo_data: Query<&mut GizmoData>,
    mut chunk_manager_events: EventReader<WorldChunkUpdateEvent>,
) {
    
    let mut gizmo_data = match gizmo_data.single_mut() {
        Ok(gizmo_data) => gizmo_data,
        Err(_) => {
            warn!("Gizmo data not found for update_gizmo!");
            return;
        },
    };

    let mut dirty = false;

    for ev in controller_position_change_ev.read() {
        gizmo_data.last_controller_pos = ev.new_pos;
        dirty = true;
    }

    for ev in controller_looking_dir_change_ev.read() {
        gizmo_data.last_looking_dir = ev.new_looking_dir;
        dirty = true;
    }

    for _ in chunk_manager_events.read() {
        dirty = true;
    }

    if dirty && gizmo_data.last_looking_dir != Vec3::default() { 
        let world = &chunk_manager_resources.chunk_manager.get_world().world;
        let raycast_result = raycast_from_controller(gizmo_data.last_controller_pos, gizmo_data.last_looking_dir, world, &game_resources.server_block_type_storage);

        if raycast_result.collide {
            gizmo_data.block_pos = raycast_result.hitpoint.pos;
            gizmo_data.visible = true;
        } else {
            gizmo_data.visible = false;
        }
    }

    if gizmo_data.visible {
        gizmos.cuboid({
            let translation = Vec3::new(
                gizmo_data.block_pos.x as f32,
                gizmo_data.block_pos.z as f32,
                -gizmo_data.block_pos.y as f32,
            );
            Transform {
                translation,
                ..Transform::IDENTITY
            }
        },
        Color::WHITE);
    }
}

fn raycast_from_controller(
    controller_pos: Vec3, 
    controller_forward: Vec3, 
    world: &shared::entities::World, 
    server_block_type_storage: &BlockTypeStorage
) -> RaycastResult {
    // Convert bevy direction to our direction
    let mut start = Vector3::new(controller_pos.x, -controller_pos.z, controller_pos.y);
    let dir = Vector3::new(controller_forward.x, -controller_forward.z, controller_forward.y);

    let config = RaycastConfig {
        world: world,
        block_type_storage: &server_block_type_storage,
        range: 16.0,
        increment: 0.01,
    };

    start -= dir * config.increment;

    raycast(start, dir, &config)
}

fn set_block_and_update_chunk(
    chunk_manager: &mut ChunkManager, 
    set_pos: BlockPos,
    new_block: BlockID,
) {
    let chunk_pos = ChunkPos::from(set_pos);
    let block_in_chunk_pos = BlockInChunkPos::from(set_pos);

    if let Some(chunk) = chunk_manager.get_or_load_chunk(chunk_pos) {
        let mut new_block_storage = chunk.get_block_storage().clone();

        new_block_storage.set_block(block_in_chunk_pos, new_block);
        let new_chunk = Chunk::new(new_block_storage);

        chunk_manager.set_chunk(chunk_pos, new_chunk);
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