use std::collections::HashMap;

use bevy::{asset::Asset, reflect::TypePath};

use crate::{bevy_resources::TextureName, chunk_mesh_builder::TextureIndex};

use super::TextureIndexDictionary;

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct TextureIndexDictionaryAsset {
    pub textures: HashMap<TextureName, TextureIndex>,
}

impl From<TextureIndexDictionaryAsset> for TextureIndexDictionary {
    fn from(config: TextureIndexDictionaryAsset) -> Self {
        TextureIndexDictionary::new(config.textures)
    }
}