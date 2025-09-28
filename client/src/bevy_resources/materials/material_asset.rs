#[derive(Debug)]
pub enum MaterialAsset {
    TexturedCube {
        data: TexturedCubeMaterialData,
    },
    ColoredCube {
        data: ColoredCubeMaterialData,
    },
}

#[derive(Debug, Default)]
pub struct TexturedCubeMaterialData {
    pub texture_array_name: String,
}

#[derive(Debug, Default)]
pub struct ColoredCubeMaterialData {
    pub palette_name: String,
}