use bevy::prelude::*;
use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::system::{Res, ResMut},
    image::Image,
};
use shared::entities::name_to_block_id;
use thiserror::Error;

use crate::{
    bevy_render::{ColoredCubeMaterial, CutoutTexturedCubeMaterial, TexturedCubeMaterial},
    bevy_resources::{
        Dictionary, MaterialAsset, MaterialHandle, RenderDesc, RenderDescCompilationError, Storage,
        TextureAsset,
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

#[derive(Debug, Error, Clone)]
pub enum MaterialsDictionaryCompilationWarning {
    #[error("Cannot compile material: {0}, {1}")]
    CannotCompile(MaterialName, MaterialCompilationError),
}

#[derive(Debug)]
pub struct MaterialsDictionaryCompilationResult {
    pub name_to_id: Dictionary<MaterialName, MaterialId>,
    pub id_to_handle: MaterialStorage,

    /// Non-fatal issues encountered during compilation.
    pub warnings: Vec<MaterialsDictionaryCompilationWarning>,
}

impl Default for MaterialsDictionaryCompilationResult {
    fn default() -> Self {
        Self {
            name_to_id: Default::default(),
            id_to_handle: MaterialStorage::new(Vec::new()),
            warnings: Vec::default(),
        }
    }
}

impl MaterialsDictionary {
    pub fn compile(
        &self,
        textures: &mut ResMut<Assets<Image>>,
        texture_assets: &TextureDictionary,
        compiled_textures: &TextureDictionaryCompilationResult,
        textured_materials: &mut ResMut<Assets<TexturedCubeMaterial>>,
        colored_materials: &mut ResMut<Assets<ColoredCubeMaterial>>,
        cutout_materials: &mut ResMut<Assets<CutoutTexturedCubeMaterial>>,
        asset_server: Res<AssetServer>,
    ) -> MaterialsDictionaryCompilationResult {
        use MaterialsDictionaryCompilationWarning::*;
        let mut result = MaterialsDictionaryCompilationResult::default();

        self.iter()
            .enumerate()
            .for_each(|(id, (material_name, material_asset))| {
                match compile_material(
                    material_asset,
                    compiled_textures,
                    textured_materials,
                    colored_materials,
                    cutout_materials,
                ) {
                    Ok(material_handle) => {
                        result
                            .name_to_id
                            .set(material_name.to_string(), id as MaterialId);
                        result.id_to_handle.add(material_handle);
                    }
                    Err(err) => result
                        .warnings
                        .push(CannotCompile(material_name.to_string(), err)),
                }
            });

        result
    }
}

#[derive(Debug, Clone, Error)]
pub enum MaterialCompilationError {
    #[error("No such texture: {0}")]
    NoSuchTexture(String),
}

fn compile_material(
    material_asset: &MaterialAsset,
    compiled_textures: &TextureDictionaryCompilationResult,
    textured_materials: &mut ResMut<Assets<TexturedCubeMaterial>>,
    colored_materials: &mut ResMut<Assets<ColoredCubeMaterial>>,
    cutout_materials: &mut ResMut<Assets<CutoutTexturedCubeMaterial>>,
) -> Result<MaterialHandle, MaterialCompilationError> {
    let texture_name: &String = match material_asset {
        MaterialAsset::TexturedCube { data } => &data.texture_array_name,
        MaterialAsset::ColoredCube { data } => &data.palette_name,
        MaterialAsset::CutoutTexturedCube { data } => &data.texture_array_name,
    };

    let texture_id = *compiled_textures.name_to_id.get(texture_name).ok_or(
        MaterialCompilationError::NoSuchTexture(texture_name.to_string()),
    )?;

    let texture_handle = compiled_textures
        .id_to_handle
        .get_by_id(texture_id as usize)
        .ok_or(MaterialCompilationError::NoSuchTexture(
            texture_name.to_string(),
        ))?
        .clone();

    let material_handle = match material_asset {
        MaterialAsset::TexturedCube { .. } => {
            let textured_mat = TexturedCubeMaterial {
                array_texture: texture_handle,
            };

            MaterialHandle::TexturedCube(textured_materials.add(textured_mat))
        }
        MaterialAsset::ColoredCube { .. } => {
            let colored_mat = ColoredCubeMaterial {
                color_palette: texture_handle,
            };

            MaterialHandle::ColoredCube(colored_materials.add(colored_mat))
        }
        MaterialAsset::CutoutTexturedCube { .. } => {
            let cutout_textured_mat = CutoutTexturedCubeMaterial {
                array_texture: texture_handle,
            };

            MaterialHandle::CutoutTexturedCube(cutout_materials.add(cutout_textured_mat))
        }
    };

    Ok(material_handle)
}
