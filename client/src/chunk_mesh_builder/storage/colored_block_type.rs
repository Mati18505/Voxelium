#[derive(Debug, Clone, PartialEq)]
pub struct ColoredBlockType {
    pub block_type: String,

    pub is_visible: bool,
    pub is_translucent: bool,

    /// A `Vec` containing the colour palette as 32-bit integers
    pub palette: Vec<VoxelColor>,   
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct VoxelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
