use crate::chunk_mesh_builder::{TextureDictionary, TextureIndex, TextureName};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
pub struct TextureConfig {
    pub textures: HashMap<TextureName, TextureIndex>,
}

impl From<TextureConfig> for TextureDictionary {
    fn from(config: TextureConfig) -> Self {
        TextureDictionary::new(config.textures)
    }
}