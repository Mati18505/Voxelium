use shared::entities::VoxelColor;

#[derive(Debug, Clone, PartialEq)]
pub struct ColoredBlockType {
    pub block_type: String,

    pub is_visible: bool,
    pub is_translucent: bool,

    /// A `Vec` containing the colour palette as 32-bit integers
    pub palette: Vec<VoxelColor>,
}