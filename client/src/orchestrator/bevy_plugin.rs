use bevy::prelude::*;

use crate::orchestrator::systems::{
    looked_at_block::LookedAtBlockEventPlugin, position_changed::PositionChangeEventPlugin,
};

pub struct OrchestratorPlugin;

impl Plugin for OrchestratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LookedAtBlockEventPlugin, PositionChangeEventPlugin));
    }
}
