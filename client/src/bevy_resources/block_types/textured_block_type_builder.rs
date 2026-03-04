use shared::entities::BlockSide;


use crate::bevy_resources::{RenderData, RenderDesc, TexturedCubeDesc};

#[derive(Debug)]
pub struct TexturedBlockTypeBuilder {
    render_desc: RenderDesc,
}

impl TexturedBlockTypeBuilder {
    pub fn new() -> TexturedBlockTypeBuilder {
        let textured_block_type = RenderDesc::TexturedCube {
            render_data: RenderData::default(),
            textured_cube_desc: TexturedCubeDesc::default(),
        };

        TexturedBlockTypeBuilder {
            render_desc: textured_block_type,
        }
    }

    pub fn render_data(mut self, render_data: RenderData) -> Self {
        let (rd, desc) = Self::expect_textured_cube_desc(&mut self.render_desc);
        *rd = render_data;
        self
    }

    pub fn texture(mut self, block_side: BlockSide, texture_name: &str) -> Self {
        use BlockSide::*;

        let (rd, desc) = Self::expect_textured_cube_desc(&mut self.render_desc);

        match block_side {
            Back | Front | Left | Right => desc.side_texture = texture_name.to_owned(),
            Top => desc.top_texture = Some(texture_name.to_owned()),
            Bottom => desc.bottom_texture = Some(texture_name.to_owned()),
        }
        self
    }

    /// Returns `TexturedCube` [`RenderDesc`].
    pub fn build(self) -> RenderDesc {
        self.render_desc
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
