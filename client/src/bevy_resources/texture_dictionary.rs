use std::collections::HashMap;

use crate::bevy_resources::TextureName;
use crate::chunk_mesh_builder::TextureIndex;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextureDictionary {
    texture_index_dictionary: HashMap<TextureName, TextureIndex>,
}

impl TextureDictionary {
    pub fn new(textures: HashMap<TextureName, TextureIndex>) -> TextureDictionary {
        TextureDictionary {
            texture_index_dictionary: textures,
        }
    }

    #[allow(dead_code)]
    pub fn set_texture_index(&mut self, name: TextureName, index: TextureIndex) {
        self.texture_index_dictionary.insert(name, index);
    }

    pub fn get_texture_index_from_name(&self, name: &str) -> Option<TextureIndex> {
        self.texture_index_dictionary.get(name).copied()
    }
}
