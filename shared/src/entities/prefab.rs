use std::collections::HashMap;

use crate::entities::*;

/// Stores reusable asset as a collection of non empty blocks.
/// Acts as a template for creating instances of the prefab.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Prefab {
    /// Voxel positions are relative to center ({0, 0, 0} is the center of a prefab).
    voxels: Vec<Voxel>,
}

impl Prefab {
    pub fn add_voxel(&mut self, voxel: Voxel) {
        self.voxels.push(voxel);
    }

    pub fn get_voxels(&self) -> &Vec<Voxel> {
        &self.voxels
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voxel {
    pub pos: BlockPos,
    pub id: BlockID,
}