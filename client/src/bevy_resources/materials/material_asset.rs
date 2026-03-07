#[derive(Debug)]
pub enum MaterialAsset {
    Placeholder,
    TexturedCube { data: TexturedCubeMaterialData },
    ColoredCube { data: ColoredCubeMaterialData },
    CutoutTexturedCube { data: TexturedCubeMaterialData },
}

#[derive(Debug, Default)]
pub struct TexturedCubeMaterialData {
    pub texture_array_name: String,
}

#[derive(Debug, Default)]
pub struct ColoredCubeMaterialData {
    pub palette_name: String,
}
