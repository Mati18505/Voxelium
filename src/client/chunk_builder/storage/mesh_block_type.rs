use voxelium::entities::{BlockSide, BlockType};

#[derive(Debug, Clone, PartialEq)]
pub struct MeshBlockType {
    pub block_type: BlockType,

    pub is_visible: bool,
    pub is_translucent: bool,
    pub material_name: String,

    top_texture: Option<String>,
    side_texture: String,
    bottom_texture: Option<String>,
}

impl MeshBlockType {
    pub fn get_block_side_texture<'a>(&'a self, side: BlockSide) -> &'a str {
        match side {
            BlockSide::Top => self.top_texture.as_deref().unwrap_or(&self.side_texture),
            BlockSide::Bottom => self.bottom_texture.as_deref().unwrap_or(&self.side_texture),
            _ => &self.side_texture,
        }
    }
}

impl Default for MeshBlockType {
    fn default() -> Self {
        Self {
            block_type: BlockType::default(),
            is_visible: false,
            is_translucent: true,
            material_name: "default".to_owned(),

            top_texture: None,
            side_texture: "default".to_owned(),
            bottom_texture: None,
        }
    }
}
