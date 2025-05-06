use crate::chunk_builder::{TextureDictionary, TextureIndex, TextureName};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone, PartialEq)]
pub struct TextureConfig {
    pub textures: HashMap<TextureName, TextureIndex>,
}

impl Into<TextureDictionary> for TextureConfig {
    fn into(self) -> TextureDictionary {
        TextureDictionary::new(self.textures)
    }
}
