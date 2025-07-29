use std::fmt::Debug;

use crate::bevy_resources::RenderDesc;

#[derive(Debug, Clone)]
pub struct RenderBlockType {
    pub block_type: String,
    pub render_desc: RenderDesc,
}
