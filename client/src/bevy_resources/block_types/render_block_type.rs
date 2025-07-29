use std::fmt::Debug;

use crate::bevy_resources::RenderDesc;

#[derive(Debug)]
pub struct RenderBlockType {
    pub block_type: String,
    pub render_desc: RenderDesc,
}
