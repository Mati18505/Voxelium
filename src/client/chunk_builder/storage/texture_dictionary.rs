use std::collections::HashMap;

pub type TextureName = String;
pub type TextureIndex = u32;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextureDictionary {
    texture_index_dictionary: HashMap<TextureName, TextureIndex>,
}

impl TextureDictionary {
    pub fn get_texture_index_from_name(&self, name: &str) -> TextureIndex {
        *self
            .texture_index_dictionary
            .get(name)
            .expect(&format!("No texture with name {}", name))
    }
}
