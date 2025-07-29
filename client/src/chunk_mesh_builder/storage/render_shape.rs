use shared::entities::{BlockSide, VoxelColor};
use std::collections::HashMap;

use crate::chunk_mesh_builder::{MaterialId, TextureIndex};

/// Stores rendering data of BlockType.
/// Shared by multiple RenderShapes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelRenderData {
    pub visible: bool,
    pub translucent: bool,
    pub material: MaterialId,
}

impl Default for VoxelRenderData {
    fn default() -> Self {
        Self { visible: false, translucent: false, material: MaterialId::default() }
    }
}

/// Stores rendering data of particular BlockType.
#[derive(Debug, PartialEq)]
pub enum RenderShape {
    TexturedCube {
        render_data: VoxelRenderData,
        textures: HashMap<BlockSide, TextureIndex>,
    },
    ColoredCube {
        render_data: VoxelRenderData,
        palette: Vec<VoxelColor>,
    },
    Invisible,
}

impl RenderShape {
    /// Creates textured cube from default texture and HashMap<BlockSide, TextureName>
    /// Default texture is used if HashMap doesn't have texture for this block side.
    pub fn create_textured_cube(
        render_data: VoxelRenderData,
        default_texture: TextureIndex,
        mut textures: HashMap<BlockSide, TextureIndex>,
    ) -> RenderShape {
        let textures: HashMap<BlockSide, TextureIndex> = BlockSide::iterator()
            .copied()
            .map(|side| (side, textures.remove(&side).unwrap_or(default_texture)))
            .collect();

        assert_eq!(textures.len(), BlockSide::iterator().len());

        RenderShape::TexturedCube {
            render_data,
            textures,
        }
    }

    pub fn render_data(&self) -> VoxelRenderData {
        match self {
            RenderShape::TexturedCube { render_data, textures } => render_data,
            RenderShape::ColoredCube { render_data, palette } => render_data,
            RenderShape::Invisible => VoxelRenderData::default(),
        }
    }

    pub fn get_vertex_attribute(&self, side: BlockSide) -> u32 {
        match self {
            RenderShape::TexturedCube { render_data, textures } => textures.get(&side).unwrap().copy(),
            // TODO: get palette index from index stored in Block
            RenderShape::ColoredCube { render_data, palette } => palette.get(0).copied(),
            RenderShape::Invisible => 0,
        }
    }
}
