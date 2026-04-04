use std::sync::Arc;

use bevy::prelude::*;
use shared::entities::*;

use crate::{
    asset_plugin::VoxelAssets,
    bevy_render::{ColoredCubeMaterial, CutoutTexturedCubeMaterial, TexturedCubeMaterial},
    bevy_resources::{
        BevyBlockTypeStorageAsset, MaterialsDictAsset, MaterialsDictionaryCompilationResult,
        RenderDescDictAsset, Storage, TextureDictAsset, TextureDictionaryCompilationResult,
    },
    bevy_types::{AppStates, GameResources},
    chunk_mesh_builder::RenderShape,
};

pub struct ResourcesPlugin;
impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SourceTextures>()
            .init_resource::<MaterialsResource>()
            .init_resource::<RenderShapeStorageRes>()
            .add_systems(
                OnEnter(AppStates::Compile),
                (
                    init_block_registry,
                    compile_texture_dictionary,
                    compile_material_dictionary,
                    compile_render_desc_dictionary,
                    create_game_resources,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
struct SourceTextures(TextureDictionaryCompilationResult);

#[derive(Resource, Default)]
struct MaterialsResource(MaterialsDictionaryCompilationResult);

#[derive(Resource, Default)]
struct RenderShapeStorageRes(Arc<Storage<RenderShape>>);

fn init_block_registry(
    voxel_assets: Res<VoxelAssets>,
    server_block_type_assets: Res<Assets<BevyBlockTypeStorageAsset>>,
) {
    let server_block_type_storage_asset = server_block_type_assets
        .get(&voxel_assets.server_blocks)
        .expect("Failed to get server_block_type_storage asset")
        .to_owned();

    init_block_names(server_block_type_storage_asset.into());
}

fn compile_texture_dictionary(
    mut result: ResMut<SourceTextures>,
    asset_server: ResMut<AssetServer>,
    voxel_assets: Res<VoxelAssets>,
    texture_dict_asset: Res<Assets<TextureDictAsset>>,
) {
    let texture_dictionary_asset: &TextureDictAsset = texture_dict_asset
        .get(&voxel_assets.texture_dict_asset)
        .unwrap();

    dbg!(&texture_dictionary_asset.0);

    let texture_dictionary_compilation_result = texture_dictionary_asset.0.compile(asset_server);

    dbg!(&texture_dictionary_compilation_result);
    result.0 = texture_dictionary_compilation_result;
}

fn compile_material_dictionary(
    mut result: ResMut<MaterialsResource>,
    mut placeholder_materials: ResMut<Assets<StandardMaterial>>,
    mut textured_materials: ResMut<Assets<TexturedCubeMaterial>>,
    mut colored_materials: ResMut<Assets<ColoredCubeMaterial>>,
    mut cutout_materials: ResMut<Assets<CutoutTexturedCubeMaterial>>,
    materials_dict_asset: Res<Assets<MaterialsDictAsset>>,
    voxel_assets: Res<VoxelAssets>,
    so_textures: Res<SourceTextures>,
) {
    let materials_dict_asset: &MaterialsDictAsset = materials_dict_asset
        .get(&voxel_assets.materials_dict_asset)
        .unwrap();

    dbg!(&materials_dict_asset.0);

    let material_compilation_result = materials_dict_asset.0.compile(
        &so_textures.0,
        &mut placeholder_materials,
        &mut textured_materials,
        &mut colored_materials,
        &mut cutout_materials,
    );

    for warning in &material_compilation_result.warnings {
        warn!("{warning}");
    }

    dbg!(&material_compilation_result);

    result.0 = material_compilation_result
}

fn compile_render_desc_dictionary(
    mut result: ResMut<RenderShapeStorageRes>,
    render_desc_dict_asset: Res<Assets<RenderDescDictAsset>>,
    voxel_assets: Res<VoxelAssets>,
    materials_res: Res<MaterialsResource>,
    texture_dictionary_asset: Res<Assets<TextureDictAsset>>,
) {
    let render_desc_dict_asset: &RenderDescDictAsset = render_desc_dict_asset
        .get(&voxel_assets.render_desc_storage_res)
        .unwrap();

    let texture_dictionary_asset: &TextureDictAsset = texture_dictionary_asset
        .get(&voxel_assets.texture_dict_asset)
        .unwrap();

    let compilation_out = render_desc_dict_asset.0.compile(
        &materials_res.0.name_to_id,
        &materials_res.0.id_to_texture_name,
        &texture_dictionary_asset.0,
    );
    let render_shape_storage = Arc::new(compilation_out.storage);

    for warning in compilation_out.warnings {
        warn!("{warning}");
    }

    dbg!(&render_shape_storage.iter().len());

    result.0 = render_shape_storage;
}

fn create_game_resources(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppStates>>,
    voxel_assets: Res<VoxelAssets>,
    material_compilation_result: Res<MaterialsResource>,
    render_shape_storage: Res<RenderShapeStorageRes>,
    server_block_type_assets: Res<Assets<BevyBlockTypeStorageAsset>>,
) {
    let material_storage = Arc::new(material_compilation_result.0.id_to_handle.clone());
    let render_shape_storage = render_shape_storage.0.clone();

    let server_block_type_storage_asset = server_block_type_assets
        .get(&voxel_assets.server_blocks)
        .expect("Failed to get server_block_type_storage asset");
    let server_block_type_storage: Arc<BlockTypeStorage> =
        Arc::new(server_block_type_storage_asset.clone().into());

    commands.insert_resource(GameResources {
        server_block_type_storage,
        material_storage,
        render_shape_storage,
    });

    next_state.set(AppStates::InGame);
}
