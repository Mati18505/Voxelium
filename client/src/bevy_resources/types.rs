use bevy::prelude::*;
use bevy::{
    asset::{AssetEvent, AssetServer, Assets, Handle},
    ecs::{
        event::EventReader,
        system::{Res, ResMut},
    },
    image::Image,
};
use shared::entities::name_to_block_id;
use thiserror::Error;

use crate::{
    bevy_render::{ColoredCubeMaterial, TexturedCubeMaterial},
    bevy_resources::{
        ColoredCubeMaterialData, Dictionary, MaterialAsset, MaterialHandle, RenderDesc,
        RenderDescCompilationError, Storage, TextureAsset, TexturedCubeMaterialData,
    },
    chunk_mesh_builder::{MaterialId, RenderShape, TextureIndex},
};

pub type MaterialName = String;
pub type TextureName = String;
pub type BlockTypeName = String;
pub type TextureId = u32;

// Assets
pub type MaterialsDictionary = Dictionary<MaterialName, MaterialAsset>;
pub type RenderDescDictionary = Dictionary<BlockTypeName, RenderDesc>;
pub type TextureDictionary = Dictionary<TextureName, TextureAsset>;

/// Each [`TextureAsset::TextureArray `] have one [`TextureIndexDictionary`].
pub type TextureIndexDictionary = Dictionary<TextureName, TextureIndex>;

// Resources
pub type MaterialStorage = Storage<MaterialHandle>;
pub type RenderShapeStorage = Storage<RenderShape>;
pub type TextureIdStorage = Storage<Handle<Image>>;

#[derive(Debug, Error, Clone)]
pub enum RenderDescDictionaryCompilationWarning {
    #[error("Cannot compile block type: {0}, {1}")]
    CannotCompile(BlockTypeName, RenderDescCompilationError),
}

#[derive(Debug, Clone)]
pub struct RenderDescDictionaryCompilationOutput {
    /// Generated output.
    pub storage: RenderShapeStorage,
    /// Non-fatal issues encountered during compilation.
    pub warnings: Vec<RenderDescDictionaryCompilationWarning>,
}

impl RenderDescDictionary {
    /// Creates [`RenderShapeStorage`] from [`RenderDescDictionary`], by compiling each [`RenderShape`] from [`RenderDesc`].
    /// Items in `RenderShapeStorage` are in order defined by `block_registry`.
    pub fn compile(
        &self,
        texture_dictionary: &TextureIndexDictionary,
        material_name_to_id: &Dictionary<MaterialName, MaterialId>,
    ) -> RenderDescDictionaryCompilationOutput {
        use RenderDescDictionaryCompilationWarning::*;
        let mut warnings: Vec<RenderDescDictionaryCompilationWarning> = vec![];

        let mut compiled = Vec::new();

        for (block_type_name, render_desc) in self.iter() {
            let block_id = name_to_block_id(block_type_name);

            match render_desc.compile(texture_dictionary, material_name_to_id) {
                Ok(c) => compiled.push((block_id, c)),
                Err(e) => warnings.push(CannotCompile(block_type_name.to_string(), e)),
            }
        }

        compiled.sort_by_key(|(id, _)| *id);
        let storage =
            RenderShapeStorage::new(compiled.into_iter().map(|(_, shape)| shape).collect());

        RenderDescDictionaryCompilationOutput { storage, warnings }
    }
}

#[derive(Debug, Default)]
pub struct TextureDictionaryCompilationResult {
    pub name_to_id: Dictionary<TextureName, TextureId>,
    pub id_to_handle: TextureIdStorage,

    // For texture array creation.
    pub asset_id_to_id: Dictionary<AssetId<Image>, TextureId>,
    pub id_to_name: Dictionary<TextureId, TextureName>,
}

impl TextureDictionary {
    pub fn compile(&self, asset_server: Res<AssetServer>) -> TextureDictionaryCompilationResult {
        let mut result = TextureDictionaryCompilationResult::default();

        self.iter()
            .enumerate()
            .for_each(|(id, (texture_name, texture_asset))| {
                let path: &str = match texture_asset {
                    TextureAsset::TextureArray { data } => &data.path,
                    TextureAsset::Palette { data } => &data.path,
                };
                let loading = asset_server.load(path);

                result
                    .name_to_id
                    .set(texture_name.to_string(), id as TextureId);
                result.asset_id_to_id.set(loading.id(), id as TextureId);
                result.id_to_handle.add(loading);
                result
                    .id_to_name
                    .set(id as TextureId, texture_name.to_string());
            });

        result
    }
}

#[derive(Debug)]
pub struct MaterialsDictionaryCompilationResult {
    pub name_to_id: Dictionary<MaterialName, MaterialId>,
    pub id_to_handle: MaterialStorage,
}

impl Default for MaterialsDictionaryCompilationResult {
    fn default() -> Self {
        Self {
            name_to_id: Default::default(),
            id_to_handle: MaterialStorage::new(Vec::new()),
        }
    }
}

impl MaterialsDictionary {
    pub fn compile(
        &self,
        mut textures: &mut ResMut<Assets<Image>>,
        texture_assets: &TextureDictionary,
        compiled_textures: &TextureDictionaryCompilationResult,
        textured_materials: &mut ResMut<Assets<TexturedCubeMaterial>>,
        colored_materials: &mut ResMut<Assets<ColoredCubeMaterial>>,
        asset_server: Res<AssetServer>,
    ) -> MaterialsDictionaryCompilationResult {
        let mut result = MaterialsDictionaryCompilationResult::default();
        self.iter()
            .enumerate()
            .for_each(|(id, (material_name, material_asset))| {
                let texture_name: &String = match material_asset {
                    MaterialAsset::TexturedCube { data } => &data.texture_array_name,
                    MaterialAsset::ColoredCube { data } => &data.palette_name,
                };
                let texture_id = compiled_textures.name_to_id.get(texture_name).unwrap();
                let texture_handle = compiled_textures
                    .id_to_handle
                    .get_by_id(*texture_id as usize)
                    .unwrap()
                    .clone();

                let material_handle = match material_asset {
                    MaterialAsset::TexturedCube { data } => {
                        let textured_mat = TexturedCubeMaterial {
                            array_texture: texture_handle,
                        };

                        MaterialHandle::TexturedCube(textured_materials.add(textured_mat))
                    }
                    MaterialAsset::ColoredCube { data } => {
                        let colored_mat = ColoredCubeMaterial {
                            color_palette: texture_handle,
                        };
                        MaterialHandle::ColoredCube(colored_materials.add(colored_mat))
                    }
                };

                result
                    .name_to_id
                    .set(material_name.to_string(), id as MaterialId);
                result.id_to_handle.add(material_handle);
            });

        result
    }
}

// impl From<TextureDictionary> for Vec<(String, TextureId)> {
//     fn from(resource: BevyBlockTypeStorageResource) -> Self {
//         resource
//             .blocks
//             .into_iter()
//             .enumerate()
//             .map(|(i, e)| (e.name, i as BlockID))
//             .collect()
//     }
// }
