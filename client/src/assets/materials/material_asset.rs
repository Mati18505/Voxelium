use bevy::prelude::*;

use crate::bevy_resources::MaterialsDictionary;

#[derive(Debug)]
pub enum MaterialAsset {
    Placeholder,
    Textured { data: TexturedCubeMaterialData },
    Colored { data: ColoredCubeMaterialData },
    CutoutTextured { data: TexturedCubeMaterialData },
}

#[derive(Debug, Default)]
pub struct TexturedCubeMaterialData {
    pub texture_array_name: String,
}

#[derive(Debug, Default)]
pub struct ColoredCubeMaterialData {
    pub palette_name: String,
}

#[derive(Debug, Asset, TypePath)]
pub struct MaterialsDictAsset(pub MaterialsDictionary);
