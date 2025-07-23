use bevy::prelude::*;

use crate::{bevy_types::AppStates, orchestrator::systems::editor::EditorPlugin};

use super::{
    events::LookedAtBlockChangedEvent,
    systems::looked_at_block::{initialize_looked_at_block, update_looked_at_block},
};

pub struct OrchestratorPlugin;

impl Plugin for OrchestratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EditorPlugin)
            .add_systems(Startup, initialize_looked_at_block)
            .add_systems(
                Update,
                update_looked_at_block.run_if(in_state(AppStates::InGame)),
            )
            .add_event::<LookedAtBlockChangedEvent>();
    }
}
