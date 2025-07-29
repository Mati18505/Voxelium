use std::collections::HashMap;
use shared::entities::BlockSide;

use crate::chunk_mesh_builder::TextureName;

#[derive(Debug, Clone, PartialEq)]
pub enum RenderShape {
    TexturedCube {
        textures: HashMap<BlockSide, TextureName>,
    },
    ColoredCube {
        palette: Vec<VoxelColor>,
    },
    Invisible,
}

impl RenderShape {
    /// Creates textured cube from default texture and HashMap<BlockSide, TextureName>
    /// Default texture is used if HashMap doesn't have texture for this block side.
    pub fn create_textured_cube(default_texture: TextureName, mut textures: HashMap<BlockSide, TextureName>) -> RenderShape {
        let textures: HashMap<BlockSide, TextureName> = BlockSide::iterator().copied()
            .map(|side| (side, textures.remove(&side).unwrap_or(default_texture.clone())))
            .collect();

        assert_eq!(textures.len(), BlockSide::iterator().len());

        RenderShape::TexturedCube { textures }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct VoxelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
