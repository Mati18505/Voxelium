use bevy::prelude::*;
use std::collections::HashMap;

use crate::bevy_resources::{TextureDictionary, TextureName};
use crate::chunk_mesh_builder::TextureIndex;

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
pub struct TextureConfig {
    pub textures: HashMap<TextureName, TextureIndex>,
}

impl From<TextureConfig> for TextureDictionary {
    fn from(config: TextureConfig) -> Self {
        TextureDictionary::new(config.textures)
    }
}
