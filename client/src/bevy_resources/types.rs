use bevy::prelude::*;
use bevy::{
    asset::{AssetServer, Assets, Handle},
    color::palettes::css::GRAY,
    ecs::system::ResMut,
    image::{Image, ImageArrayLayout, ImageLoaderSettings},
};
use shared::entities::{iterate_over_block_registry, name_to_block_id, BlockID};
use thiserror::Error;

use crate::bevy_resources::RenderDescCompileCtx;
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
        material_id_to_texture_name: &Dictionary<MaterialId, TextureName>,
        texture_asset_dictionary: &TextureDictionary,
    ) -> RenderDescDictionaryCompilationOutput {
        use RenderDescDictionaryCompilationWarning::*;
        let mut out = RenderDescDictionaryCompilationOutput::default();

        for (render_desc_name, _render_desc) in self.iter() {
            if render_desc_name != "air" && name_to_block_id(render_desc_name) == BlockID::default()
            {
                out.warnings
                    .push(NoCorrespondingBlockInRegistry(render_desc_name.to_string()));
            }
        }

        let mut block_registry: Vec<_> = iterate_over_block_registry().collect();
        block_registry.sort_by_key(|(_, block_id)| *block_id);

        for (block_type_name, _block_id) in block_registry {
            let render_shape = if let Some(render_desc) = self.get(block_type_name) {
                let ctx = RenderDescCompileCtx {
                    material_name_to_id,
                    material_id_to_texture_name,
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

            out.storage.add(render_shape);
        }

        out
    }
}

#[derive(Debug, Default)]
pub struct TextureDictionaryCompilationResult {
    pub name_to_id: Dictionary<TextureName, TextureId>,
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

#[derive(Debug, Error, Clone)]
pub enum MaterialsDictionaryCompilationWarning {
    #[error("Cannot compile material: {0}, {1}")]
    CannotCompile(MaterialName, MaterialCompilationError),
}

#[derive(Debug, Default)]
pub struct MaterialsDictionaryCompilationResult {
    pub name_to_id: Dictionary<MaterialName, MaterialId>,
    pub id_to_handle: MaterialStorage,

    // For render desc compilation.
    // Should be set for every material that uses texture.
    // No value if material doesn't use texture.
    pub id_to_texture_name: Dictionary<MaterialId, TextureName>,

    /// Non-fatal issues encountered during compilation.
    pub warnings: Vec<MaterialsDictionaryCompilationWarning>,
}

impl MaterialsDictionary {
    pub fn compile(
        &self,
        compiled_textures: &TextureDictionaryCompilationResult,
        placeholder_materials: &mut ResMut<Assets<StandardMaterial>>,
        textured_materials: &mut ResMut<Assets<TexturedCubeMaterial>>,
        colored_materials: &mut ResMut<Assets<ColoredCubeMaterial>>,
        cutout_materials: &mut ResMut<Assets<CutoutTexturedCubeMaterial>>,
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
                    MaterialAsset::TexturedCube { data } => Some(&data.texture_array_name),
                    MaterialAsset::ColoredCube { data } => Some(&data.palette_name),
                    MaterialAsset::CutoutTexturedCube { data } => Some(&data.texture_array_name),
                }
                .cloned();

                if let Some(texture_name) = &maybe_texture_name {
                    result
                        .id_to_texture_name
                        .set(id as MaterialId, texture_name.to_string());
                }

                match compile_material(
                    material_asset,
                    compiled_textures,
                    textured_materials,
                    colored_materials,
                    cutout_materials,
                    placeholder_materials,
                    maybe_texture_name,
                ) {
                    Ok(material_handle) => {
                        result.id_to_handle.add(material_handle);
                    }
                    Err(err) => {
                        let placeholder = result.id_to_handle.get_by_id(0).cloned().unwrap();

                        result.id_to_handle.add(placeholder);
                        result
                            .warnings
                            .push(CannotCompile(material_name.to_string(), err));
                    }
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
    placeholder_materials: &mut ResMut<Assets<StandardMaterial>>,
    maybe_texture_name: Option<TextureName>,
) -> Result<MaterialHandle, MaterialCompilationError> {
    match maybe_texture_name {
        Some(texture_name) => compile_material_with_texture(
            material_asset,
            compiled_textures,
            textured_materials,
            colored_materials,
            cutout_materials,
            texture_name,
        ),
        None => compile_material_no_texture(material_asset, placeholder_materials),
    }
}

fn compile_material_no_texture(
    material_asset: &MaterialAsset,
    placeholder_materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Result<MaterialHandle, MaterialCompilationError> {
    let material_handle = match material_asset {
        MaterialAsset::Placeholder => {
            let placeholder = StandardMaterial {
                base_color: Color::srgba(0.54, 0.0, 0.54, 1.0),
                ..Default::default()
            };
            MaterialHandle::Placeholder(placeholder_materials.add(placeholder))
        }
        MaterialAsset::TexturedCube { data } => unreachable!(),
        MaterialAsset::ColoredCube { data } => unreachable!(),
        MaterialAsset::CutoutTexturedCube { data } => unreachable!(),
    };

    Ok(material_handle)
}

fn compile_material_with_texture(
    material_asset: &MaterialAsset,
    compiled_textures: &TextureDictionaryCompilationResult,
    textured_materials: &mut ResMut<Assets<TexturedCubeMaterial>>,
    colored_materials: &mut ResMut<Assets<ColoredCubeMaterial>>,
    cutout_materials: &mut ResMut<Assets<CutoutTexturedCubeMaterial>>,
    texture_name: TextureName,
) -> Result<MaterialHandle, MaterialCompilationError> {
    let texture_id = *compiled_textures.name_to_id.get(&texture_name).ok_or(
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
        MaterialAsset::Placeholder => unreachable!(),
    };

    Ok(material_handle)
}
