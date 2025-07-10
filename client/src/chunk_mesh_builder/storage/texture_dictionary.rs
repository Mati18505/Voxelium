use std::collections::HashMap;

pub type TextureName = String;
pub type TextureIndex = u32;

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

    pub fn set_texture_index(&mut self, name: TextureName, index: TextureIndex) {
        self.texture_index_dictionary.insert(name, index);
    }

    pub fn get_texture_index_from_name(&self, name: &str) -> Option<TextureIndex> {
        self.texture_index_dictionary.get(name).map(|e| e.clone())
    }
}
