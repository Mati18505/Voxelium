use bevy::{asset::Asset, reflect::TypePath};

use crate::bevy_resources::{TextureDictionary, TextureIndexDictionary};

#[derive(Debug, Clone)]
pub enum TextureAsset {
    TextureArray { data: TextureArrayData },
    Palette { data: PaletteData },
}

#[derive(Debug, Default, Clone)]
pub struct TextureArrayData {
    pub path: String,
    pub textures: TextureIndexDictionary,
}

#[derive(Debug, Default, Clone)]
pub struct PaletteData {
    pub path: String,
    pub len: usize,
}

#[derive(Debug, Asset, TypePath)]
pub struct TextureDictAsset(pub TextureDictionary);

impl From<TextureDictAsset> for TextureDictionary {
    fn from(asset: TextureDictAsset) -> Self {
        asset.0
    }
}
