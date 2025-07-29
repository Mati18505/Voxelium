use shared::entities::{BlockSide, VoxelColor};
use std::collections::HashMap;

use crate::chunk_mesh_builder::{MaterialId, TextureIndex, TextureName};

/// Stores rendering data of BlockType.
/// Shared by multiple RenderShapes.
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelRenderData {
    pub visible: bool,
    pub material: MaterialId,
}

/// Stores rendering data of particular BlockType.
#[derive(Debug, Clone, PartialEq)]
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
        default_texture: TextureName,
        mut textures: HashMap<BlockSide, TextureName>,
    ) -> RenderShape {
        let textures: HashMap<BlockSide, TextureName> = BlockSide::iterator()
            .copied()
            .map(|side| {
                (
                    side,
                    textures.remove(&side).unwrap_or(default_texture.clone()),
                )
            })
            .collect();

        assert_eq!(textures.len(), BlockSide::iterator().len());

        RenderShape::TexturedCube {
            render_data,
            textures,
        }
    }
}