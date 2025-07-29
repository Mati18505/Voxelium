use std::fmt::Debug;

use shared::entities::BlockID;
use crate::chunk_mesh_builder::RenderShape;

#[derive(Debug, Default)]
pub struct RenderShapeStorage {
    /// Map from index ([`BlockID`]) to RenderShape.
    render_shapes: Vec<RenderShape>,
}

impl RenderShapeStorage {
    pub fn new(render_shapes: Vec<RenderShape>) -> Self {
        Self { render_shapes }
    }

    pub fn get_render_shape_from_id(&self, id: BlockID) -> Option<&RenderShape> {
        self.render_shapes.get(id)
    }
}

