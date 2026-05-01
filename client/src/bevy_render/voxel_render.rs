use bevy::prelude::*;

use super::{EntitiesManagerPlugin, voxel_materials::VoxelMaterialsPlugin};

pub struct VoxelRenderPlugin;
impl Plugin for VoxelRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VoxelMaterialsPlugin);
        app.add_plugins(EntitiesManagerPlugin);
    }
}
