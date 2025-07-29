use shared::entities::BlockSide;
use std::collections::HashMap;

use crate::chunk_mesh_builder::{RenderShape, VoxelRenderData};

use crate::bevy_resources::{
    MaterialName, RenderBlockType, RenderData, RenderDesc, TextureName, TexturedCubeDesc,
};

#[derive(Debug)]
pub struct TexturedBlockTypeBuilder {
    block_type: RenderBlockType,
}

impl TexturedBlockTypeBuilder {
    pub fn new(block_type: &str) -> TexturedBlockTypeBuilder {
        let textured_block_type = RenderBlockType {
            block_type: block_type.to_string(),
            render_desc: RenderDesc::TexturedCube {
                render_data: RenderData::default(),
                textured_cube_desc: TexturedCubeDesc::default(),
            },
        };

        TexturedBlockTypeBuilder {
            block_type: textured_block_type,
        }
    }

    pub fn visible(mut self, visible: bool) -> Self {
        let (rd, desc) = Self::expect_textured_cube_desc(&mut self.block_type.render_desc);
        rd.visible = visible;
        self
    }

    pub fn translucent(mut self, translucent: bool) -> Self {
        let (rd, desc) = Self::expect_textured_cube_desc(&mut self.block_type.render_desc);
        rd.translucent = translucent;
        self
    }

    pub fn material(mut self, material_name: MaterialName) -> Self {
        let (rd, desc) = Self::expect_textured_cube_desc(&mut self.block_type.render_desc);
        rd.material = material_name;
        self
    }

    pub fn texture(mut self, block_side: BlockSide, texture_name: &str) -> Self {
        use BlockSide::*;

        let (rd, desc) = Self::expect_textured_cube_desc(&mut self.block_type.render_desc);

        match block_side {
            Back | Front | Left | Right => desc.side_texture = texture_name.to_owned(),
            Top => desc.top_texture = Some(texture_name.to_owned()),
            Bottom => desc.bottom_texture = Some(texture_name.to_owned()),
        }
        self
    }

    /// Returns [`RenderBlockType`] with `TexturedCube` [`RenderDesc`].
    pub fn build(self) -> RenderBlockType {
        self.block_type
    }

    fn expect_textured_cube_desc(
        desc: &mut RenderDesc,
    ) -> (&mut RenderData, &mut TexturedCubeDesc) {
        match desc {
            RenderDesc::TexturedCube {
                render_data,
                textured_cube_desc,
            } => (render_data, textured_cube_desc),
            _ => unreachable!(),
        }
    }
}
