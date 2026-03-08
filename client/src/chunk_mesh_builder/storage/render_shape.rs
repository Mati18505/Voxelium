use shared::entities::BlockSide;
use std::collections::HashMap;

use crate::chunk_mesh_builder::{ColorIndex, MaterialId, StorageIndex, TextureIndex};

/// Stores rendering data of BlockType.
/// Shared by multiple RenderShapes.
#[derive(Debug, Clone, Copy)]
pub struct VoxelRenderData {
    pub visible: bool,
    pub translucent: bool,
    pub material: MaterialId,
}

impl VoxelRenderData {
    pub const fn const_default() -> Self {
        Self {
            visible: false,
            translucent: true,
            material: 0,
        }
    }

    pub const fn placeholder() -> Self {
        Self {
            visible: true,
            translucent: false,
            material: 0,
        }
    }
}

/// Stores rendering data of particular BlockType.
#[derive(Debug, Clone)]
pub enum RenderShape {
    TexturedCube {
        render_data: VoxelRenderData,
        textures: HashMap<BlockSide, TextureIndex>,
    },
    ColoredCube {
        render_data: VoxelRenderData,
        color_index: ColorIndex,
    },
    Placeholder,
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

    pub fn render_data(&self) -> &VoxelRenderData {
        static DEFAULT_RENDER_DATA: VoxelRenderData = VoxelRenderData::const_default();
        static PLACEHOLDER_RENDER_DATA: VoxelRenderData = VoxelRenderData::placeholder();

        match self {
            RenderShape::TexturedCube {
                render_data,
                textures: _,
            } => render_data,
            RenderShape::ColoredCube {
                render_data,
                color_index: _,
            } => render_data,
            RenderShape::Placeholder => &PLACEHOLDER_RENDER_DATA,
            RenderShape::Invisible => &DEFAULT_RENDER_DATA,
        }
    }

    pub fn get_storage_index(&self, side: BlockSide) -> Option<StorageIndex> {
        match self {
            RenderShape::TexturedCube {
                render_data: _,
                textures,
            } => Some(*textures.get(&side).unwrap()),
            RenderShape::ColoredCube {
                render_data: _,
                color_index,
            } => Some(*color_index),
            RenderShape::Placeholder => Some(0),
            _ => None,
        }
    }
}
