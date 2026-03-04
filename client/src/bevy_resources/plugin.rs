use std::sync::Arc;

use bevy::prelude::*;
use shared::entities::*;

use crate::{
    bevy_render::{ColoredCubeMaterial, CutoutTexturedCubeMaterial, TexturedCubeMaterial},
    bevy_resources::{
        BevyBlockTypeStorageAsset, MaterialsDictAsset, MaterialsDictionary, RenderDescDictAsset,
        RenderDescDictionary, TextureAsset, TextureDictAsset, TextureDictionary,
        TextureDictionaryCompilationResult, TextureIndexDictionary,
    },
    bevy_types::{AppStates, GameResources},
    VoxelAssets,
};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum ResourcesCompilingState {
    #[default]
    CompilingTextures,
    LoadingTextures,
    CompilingRest,
}

pub struct ResourcesPlugin;
impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ResourcesCompilingState>()
            .init_resource::<SourceTextures>()
            .add_systems(OnEnter(AppStates::Compile), compile_assets)
            .add_systems(
                OnEnter(ResourcesCompilingState::CompilingRest),
                compile_rest,
            )
            .add_systems(
                Update,
                check_all_textures_loaded
                    .run_if(in_state(ResourcesCompilingState::LoadingTextures)),
            )
            .add_systems(
                Update,
                create_texture_arrays.run_if(in_state(ResourcesCompilingState::LoadingTextures)),
            );
    }
}

fn compile_assets(
    texture_dict_asset: ResMut<Assets<TextureDictAsset>>,
    voxel_assets: Res<VoxelAssets>,
    asset_server: Res<AssetServer>,
    textures_out: ResMut<SourceTextures>,
    mut next_state: ResMut<NextState<ResourcesCompilingState>>,
) {
    load_textures(texture_dict_asset, voxel_assets, asset_server, textures_out);
    next_state.set(ResourcesCompilingState::LoadingTextures);
}

fn load_textures(
    texture_dict_asset: ResMut<Assets<TextureDictAsset>>,
    voxel_assets: Res<VoxelAssets>,
    asset_server: Res<AssetServer>,
    mut textures_out: ResMut<SourceTextures>,
) {
    let texture_dictionary_asset: &TextureDictAsset = texture_dict_asset
        .get(&voxel_assets.texture_dict_asset)
        .unwrap();
    let texture_dictionary: Arc<TextureDictionary> = Arc::new(texture_dictionary_asset.0.clone());

    dbg!(&texture_dictionary);

    let result = texture_dictionary.compile(asset_server);
    dbg!(&result);

    textures_out.textures = result;
}

fn compile_rest(
    mut commands: Commands,
    mut placeholder_materials: ResMut<Assets<StandardMaterial>>,
    mut textured_materials: ResMut<Assets<TexturedCubeMaterial>>,
    mut colored_materials: ResMut<Assets<ColoredCubeMaterial>>,
    mut cutout_materials: ResMut<Assets<CutoutTexturedCubeMaterial>>,
    mut textures: ResMut<Assets<Image>>,
    texture_dict_asset: ResMut<Assets<TextureDictAsset>>,
    mut render_desc_dict_asset: ResMut<Assets<RenderDescDictAsset>>,
    mut materials_dict_asset: ResMut<Assets<MaterialsDictAsset>>,
    server_block_type_assets: Res<Assets<BevyBlockTypeStorageAsset>>,
    voxel_assets: Res<VoxelAssets>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppStates>>,
    loaded_textures: Res<SourceTextures>,
) {
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

    let maybe_opaque_id = loaded_textures
        .textures
        .name_to_id
        .get(&"opaque".to_string());
    let maybe_opaque = loaded_textures
        .textures
        .id_to_handle
        .get_by_id(*maybe_opaque_id.unwrap() as usize);

    // TODO: make this flexible.
    let texture_index_dictionary: &TextureIndexDictionary =
        match texture_dictionary.get(&"opaque".to_string()).unwrap() {
            TextureAsset::TextureArray { data } => &data.textures,
            TextureAsset::Palette { data } => unimplemented!(),
        };

    let material_compilation_result = materials_dict.compile(
        &mut textures,
        &texture_dictionary,
        &loaded_textures.textures,
        &mut placeholder_materials,
        &mut textured_materials,
        &mut colored_materials,
        &mut cutout_materials,
        asset_server,
    );

    for warning in &material_compilation_result.warnings {
        warn!("{warning}");
    }

    dbg!(&material_compilation_result);

    let texture_index_dictionary = Arc::new(texture_index_dictionary.clone());
    let material_storage = Arc::new(material_compilation_result.id_to_handle);

    init_block_names(server_block_type_storage_asset.into());

    let compilation_out = render_desc_dict.compile(
        &texture_index_dictionary,
        &material_compilation_result.name_to_id,
    );
    let render_shape_storage = Arc::new(compilation_out.storage);

    for warning in compilation_out.warnings {
        warn!("{warning}");
    }

    dbg!(&render_shape_storage.iter().len());

    commands.insert_resource(GameResources {
        render_desc_dict,
        server_block_type_storage,
        texture_index_dictionary,
        material_storage,
        render_shape_storage,
    });

    next_state.set(AppStates::InGame);
}

#[derive(Resource, Default)]
struct SourceTextures {
    textures: TextureDictionaryCompilationResult,
}

fn check_all_textures_loaded(
    textures: Res<SourceTextures>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<ResourcesCompilingState>>,
) {
    // TODO: handle loading fail
    if textures
        .textures
        .id_to_handle
        .iter()
        .any(|h| !asset_server.get_load_state(h).unwrap().is_loaded())
    {
        return;
    }

    next_state.set(ResourcesCompilingState::CompilingRest);
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
