use crate::bevy_resources::TextureIndexDictionary;

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
