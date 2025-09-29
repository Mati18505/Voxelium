use crate::bevy_resources::TextureIndexDictionary;

#[derive(Debug)]
pub enum TextureAsset {
    TextureArray {
        data: TextureArrayData,
    },
    Palette {
        data: PaletteData,
    },
}

#[derive(Debug, Default)]
pub struct TextureArrayData {
    pub path: String,
    pub textures: TextureIndexDictionary,
}

#[derive(Debug, Default)]
pub struct PaletteData {
    pub path: String,
    pub len: usize,
}