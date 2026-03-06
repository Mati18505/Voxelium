use std::sync::Arc;

use bevy::prelude::*;
use shared::entities::*;

use crate::{
    bevy_render::{ColoredCubeMaterial, CutoutTexturedCubeMaterial, TexturedCubeMaterial},
    bevy_resources::{
        BevyBlockTypeStorageAsset, MaterialsDictAsset, MaterialsDictionary, RenderDescDictAsset,
        RenderDescDictionary, TextureAsset, TextureDictAsset, TextureDictionary,
        TextureDictionaryCompilationResult, TextureId, TextureIndexDictionary,
    },
    bevy_types::{AppStates, GameResources},
    VoxelAssets,
};

pub struct ResourcesPlugin;
impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SourceTextures>()
            .add_systems(OnEnter(AppStates::Compile), compile_assets)
            .add_systems(
                Update,
                (check_all_textures_loaded, create_texture_arrays)
                    .run_if(in_state(AppStates::Compile)),
            );
    }
}

fn compile_assets(
    texture_dict_asset: ResMut<Assets<TextureDictAsset>>,
    voxel_assets: Res<VoxelAssets>,
    asset_server: ResMut<AssetServer>,
    source_textures: ResMut<SourceTextures>,

    mut commands: Commands,
    mut placeholder_materials: ResMut<Assets<StandardMaterial>>,
    mut textured_materials: ResMut<Assets<TexturedCubeMaterial>>,
    mut colored_materials: ResMut<Assets<ColoredCubeMaterial>>,
    mut cutout_materials: ResMut<Assets<CutoutTexturedCubeMaterial>>,
    mut render_desc_dict_asset: ResMut<Assets<RenderDescDictAsset>>,
    mut materials_dict_asset: ResMut<Assets<MaterialsDictAsset>>,
    server_block_type_assets: Res<Assets<BevyBlockTypeStorageAsset>>,
    mut next_state: ResMut<NextState<AppStates>>,
) {
    let texture_dictionary_asset: &TextureDictAsset = texture_dict_asset
        .get(&voxel_assets.texture_dict_asset)
        .unwrap();
    let texture_dictionary: Arc<TextureDictionary> = Arc::new(texture_dictionary_asset.0.clone());

    dbg!(&texture_dictionary);

    let texture_dictionary_compilation_result = texture_dictionary.compile(asset_server);

    dbg!(&texture_dictionary_compilation_result);

    let texture_dictionary_asset: &TextureDictAsset = texture_dict_asset
        .get(&voxel_assets.texture_dict_asset)
        .unwrap();
    let texture_dictionary: Arc<TextureDictionary> = Arc::new(texture_dictionary_asset.0.clone());

    let render_desc_dict_asset: RenderDescDictAsset = render_desc_dict_asset
        .remove(&voxel_assets.render_desc_storage_res)
        .unwrap();
    let render_desc_dict: Arc<RenderDescDictionary> = Arc::new(render_desc_dict_asset.0);

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

    let material_compilation_result = materials_dict.compile(
        &texture_dictionary,
        &texture_dictionary_compilation_result,
        &mut placeholder_materials,
        &mut textured_materials,
        &mut colored_materials,
        &mut cutout_materials,
    );

    for warning in &material_compilation_result.warnings {
        warn!("{warning}");
    }

    dbg!(&material_compilation_result);

    init_block_names(server_block_type_storage_asset.into());

    let compilation_out = render_desc_dict.compile(
        &material_compilation_result.name_to_id,
        &material_compilation_result.id_to_texture_name,
        &texture_dictionary_asset.0,
    );
    let render_shape_storage = Arc::new(compilation_out.storage);

    for warning in compilation_out.warnings {
        warn!("{warning}");
    }

    dbg!(&render_shape_storage.iter().len());

    let material_storage = Arc::new(material_compilation_result.id_to_handle);

    commands.insert_resource(GameResources {
        server_block_type_storage,
        material_storage,
        render_shape_storage,
    });

    next_state.set(AppStates::InGame);
}

#[derive(Resource, Default)]
struct SourceTextures {
    textures: TextureDictionaryCompilationResult,
}

fn check_all_textures_loaded(textures: Res<SourceTextures>, asset_server: Res<AssetServer>) {
    let mut has_pending = false;

    for (texture_id, handle) in textures.textures.id_to_handle.iter().enumerate() {
        match asset_server.get_load_state(handle) {
            Some(load_state) if load_state.is_loaded() => {}
            Some(load_state) if load_state.is_failed() => {
                let texture_id = texture_id as TextureId;
                let texture_name = textures
                    .textures
                    .id_to_name
                    .get(&texture_id)
                    .cloned()
                    .unwrap();
                warn!("Texture failed to load and may render incorrectly: {texture_name}");
            }
            _ => has_pending = true,
        }
    }
}

fn create_texture_arrays(
    so_textures: Res<SourceTextures>,
    mut events: EventReader<AssetEvent<Image>>,
    voxel_assets: Res<VoxelAssets>,
    texture_dict_asset: ResMut<Assets<TextureDictAsset>>,
    mut textures: ResMut<Assets<Image>>,
) {
    let texture_dictionary_asset: &TextureDictAsset = texture_dict_asset
        .get(&voxel_assets.texture_dict_asset)
        .unwrap();
    let texture_dictionary: Arc<TextureDictionary> = Arc::new(texture_dictionary_asset.0.clone());

    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id: asset_id } = event {
            if let Some(texture_id) = so_textures.textures.asset_id_to_id.get(asset_id) {
                let texture_name = so_textures.textures.id_to_name.get(texture_id).unwrap();

                if let TextureAsset::TextureArray { data } =
                    texture_dictionary.get(texture_name).unwrap()
                {
                    info!("Creating texture array {:?}", texture_name);

                    let texture_index_dictionary = &data.textures;

                    let layers = texture_index_dictionary.iter().len();
                    create_texture_array(layers as u32, *asset_id, &mut textures);
                }
            }
        }
    }
}

fn create_texture_array(layers: u32, asset_id: AssetId<Image>, images: &mut Assets<Image>) {
    if let Some(image) = images.get_mut(asset_id) {
        image.reinterpret_stacked_2d_as_array(layers);
    }
}
