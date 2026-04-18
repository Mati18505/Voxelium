use std::collections::HashMap;

use bevy::prelude::*;
use bevy::{
    asset::{AssetServer, Handle},
    ecs::system::ResMut,
    image::{Image, ImageArrayLayout, ImageLoaderSettings},
};
use shared::entities::{BlockID, BlockRegistry, IterableBlockRegistry};
use thiserror::Error;

use crate::assets::materials::MaterialAsset;
use crate::assets::textures::TextureAsset;
use crate::bevy_resources::{RenderDescCompileCtx, RuntimeMaterial};
use crate::voxel_render_core::RenderShape;
use crate::{
    bevy_resources::{Dictionary, RenderDesc, RenderDescCompilationError, Storage},
    chunk_mesh_builder::{MaterialId, TextureIndex},
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
pub type MaterialStorage = Storage<RuntimeMaterial>;
pub type RenderShapeStorage = Vec<RenderShape>;
pub type TextureIdStorage = Storage<Handle<Image>>;

#[derive(Resource, Default)]
pub struct BlockNameToId(HashMap<String, BlockID>);

impl BlockRegistry for BlockNameToId {
    fn name_to_block_id(&self, block_name: &str) -> BlockID {
        self.0.get(block_name).cloned().unwrap_or(0)
    }
}

impl IterableBlockRegistry for BlockNameToId {
    fn iter(&self) -> impl Iterator<Item = (&str, BlockID)> {
        self.0.iter().map(|(k, v)| (k.as_str(), *v))
    }
}

impl BlockNameToId {
    pub fn new(registry: HashMap<String, BlockID>) -> Self {
        Self(registry)
    }
}

#[derive(Debug, Error, Clone)]
pub enum RenderDescDictionaryCompilationWarning {
    #[error("Cannot compile block type: {0}, {1}")]
    CannotCompile(BlockTypeName, RenderDescCompilationError),
    #[error("Cannot find corresponding render desc for block: {0}")]
    NoCorrespondingRenderDesc(BlockTypeName),
    #[error("Cannot find corresponding block for render desc: {0}")]
    NoCorrespondingBlockInRegistry(BlockTypeName),
}

#[derive(Debug, Clone, Default)]
pub struct RenderDescDictionaryCompilationOutput {
    /// Generated output.
    pub storage: RenderShapeStorage,
    /// Non-fatal issues encountered during compilation.
    pub warnings: Vec<RenderDescDictionaryCompilationWarning>,
}

impl RenderDescDictionary {
    /// Creates [`RenderShapeStorage`] from [`RenderDescDictionary`], by compiling each [`RenderShape`] from [`RenderDesc`] for every block in 'block_registry'.
    /// Items in [`RenderShapeStorage`] are in order defined by `block_registry`.
    /// Compiles only render descriptions that have corresponding entry in `block_registry'.
    pub fn compile(
        &self,
        material_name_to_id: &Dictionary<MaterialName, MaterialId>,
        runtime_materials: &MaterialStorage,
        texture_asset_dictionary: &TextureDictionary,
        registry: &(impl BlockRegistry + IterableBlockRegistry),
    ) -> RenderDescDictionaryCompilationOutput {
        use RenderDescDictionaryCompilationWarning::*;
        let mut out = RenderDescDictionaryCompilationOutput::default();

        for (render_desc_name, _render_desc) in self.iter() {
            if render_desc_name != "air"
                && registry.name_to_block_id(render_desc_name) == BlockID::default()
            {
                out.warnings
                    .push(NoCorrespondingBlockInRegistry(render_desc_name.to_string()));
            }
        }

        let mut block_registry: Vec<_> = registry.iter().collect();
        block_registry.sort_by_key(|(_, block_id)| *block_id);

        for (block_type_name, _block_id) in block_registry {
            let render_shape = if let Some(render_desc) = self.get(&block_type_name.to_owned()) {
                let ctx = RenderDescCompileCtx {
                    material_name_to_id,
                    runtime_materials,
                    texture_asset_dictionary,
                };
                match render_desc.compile(ctx) {
                    Ok(compilation_result) => compilation_result,
                    Err(e) => {
                        out.warnings
                            .push(CannotCompile(block_type_name.to_string(), e));
                        RenderShape::Placeholder
                    }
                }
            } else {
                out.warnings
                    .push(NoCorrespondingRenderDesc(block_type_name.to_string()));
                RenderShape::Placeholder
            };

            out.storage.push(render_shape);
        }

        out
    }
}

#[derive(Debug, Default)]
pub struct TextureDictionaryCompilationResult {
    pub name_to_id: Dictionary<TextureName, TextureId>,
    pub id_to_name: Dictionary<TextureId, TextureName>,
    pub id_to_handle: TextureIdStorage,
}

impl TextureDictionary {
    pub fn compile(&self, asset_server: ResMut<AssetServer>) -> TextureDictionaryCompilationResult {
        let mut result = TextureDictionaryCompilationResult::default();

        self.iter()
            .enumerate()
            .for_each(|(id, (texture_name, texture_asset))| {
                let loading = match texture_asset {
                    TextureAsset::TextureArray { data } => {
                        let layer_count = data.textures.iter().len() as u32;
                        let path = &data.path;

                        asset_server.load_with_settings(
                            path,
                            move |settings: &mut ImageLoaderSettings| {
                                settings.array_layout =
                                    Some(ImageArrayLayout::RowCount { rows: layer_count })
                            },
                        )
                    }
                    TextureAsset::Palette { data } => {
                        let path = &data.path;
                        asset_server.load(path)
                    }
                };

                result
                    .name_to_id
                    .set(texture_name.to_string(), id as TextureId);
                result.id_to_handle.add(loading);
            });

        result
    }
}

#[derive(Debug, Clone, Error)]
pub enum MaterialCompilationError {
    #[error("No such texture: {0}")]
    NoSuchTexture(String),
}

pub trait MaterialRuntimeSource {
    fn to_runtime(&self, textures_name_to_id: Dictionary<String, u32>) -> Result<RuntimeMaterial, MaterialCompilationError>;
}

#[derive(Debug, Error, Clone)]
pub enum MaterialsDictionaryCompilationWarning {
    #[error("Cannot compile material: {0}, {1}")]
    CannotCompile(MaterialName, MaterialCompilationError),
}

#[derive(Debug, Default)]
pub struct MaterialsDictionaryCompilationResult {
    pub name_to_id: Dictionary<MaterialName, MaterialId>,
    pub id_to_runtime: MaterialStorage,

    /// Non-fatal issues encountered during compilation.
    pub warnings: Vec<MaterialsDictionaryCompilationWarning>,
}

impl MaterialsDictionary {
    pub fn compile(
        &self,
        compiled_textures: &TextureDictionaryCompilationResult,
    ) -> MaterialsDictionaryCompilationResult {
        use MaterialsDictionaryCompilationWarning::*;
        let mut result = MaterialsDictionaryCompilationResult::default();

        let placeholder_name = "placeholder".to_string();
        let placeholder_item = (&placeholder_name, &MaterialAsset::Placeholder);

        // Material ID=0 is always placeholder.
        let all_materials = std::iter::once(placeholder_item).chain(self.iter());

        all_materials
            .enumerate()
            .for_each(|(id, (material_name, material_asset))| {
                result
                    .name_to_id
                    .set(material_name.to_string(), id as MaterialId);

                let maybe_texture_name: Option<String> = match material_asset {
                    MaterialAsset::Placeholder => None,
                    MaterialAsset::Textured { data } => Some(&data.texture_array_name),
                    MaterialAsset::Colored { data } => Some(&data.palette_name),
                    MaterialAsset::CutoutTextured { data } => Some(&data.texture_array_name),
                }
                .cloned();

                if let Some(texture_name) = &maybe_texture_name {
                    result
                        .id_to_texture_name
                        .set(id as MaterialId, texture_name.to_string());
                }
                match material_asset.to_runtime(compiled_textures.name_to_id) {
                    Ok(runtime_material) => {
                        result.id_to_runtime.add(runtime_material);
                    }
                    Err(err) => {
                        let placeholder = result.id_to_runtime.get_by_id(0).cloned().unwrap();

                        result.id_to_runtime.add(placeholder);
                        result
                            .warnings
                            .push(CannotCompile(material_name.to_string(), err));
                    }
                }
            });

        result
    }
}
