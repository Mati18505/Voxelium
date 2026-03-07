use bevy::prelude::*;
use shared::entities::*;

use crate::{bevy_types::AppStates, orchestrator};

pub struct GUIPlugin;
impl Plugin for GUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gui)
            .add_systems(Update, draw_gizmo.run_if(in_state(AppStates::InGame)))
            .add_observer(on_looked_at_block);
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BlockCursorData {
    visible: bool,
    block_pos: BlockPos,
}

impl Default for BlockCursorData {
    fn default() -> Self {
        Self {
            visible: false,
            block_pos: BlockPos::new(0, 0, 0),
        }
    }
}

fn setup_gui(mut commands: Commands) {
    commands.spawn((BlockCursorData::default(),));
}

fn on_looked_at_block(
    new_block: On<orchestrator::LookedAtBlockChangedEvent>,
    mut q_block_cursor_data: Query<&mut BlockCursorData>,
    state: Res<State<AppStates>>,
) {
    if !matches!(state.get(), AppStates::InGame) {
        return;
    }

    let mut block_cursor_data = match q_block_cursor_data.single_mut() {
        Ok(block_cursor_data) => block_cursor_data,
        Err(_) => {
            warn!("Gizmo data not found for update_gizmo!");
            return;
        }
    };

    block_cursor_data.visible = new_block.block_type.affect_raycast;
    block_cursor_data.block_pos = new_block.block_pos;
}

fn draw_gizmo(mut gizmos: Gizmos, q_block_cursor_data: Query<&BlockCursorData>) {
    let block_cursor_data = match q_block_cursor_data.single() {
        Ok(block_cursor_data) => block_cursor_data,
        Err(_) => {
            warn!("Gizmo data not found for update_gizmo!");
            return;
        }
    };

    if block_cursor_data.visible {
        gizmos.cube(
            {
                let translation = Vec3::new(
                    block_cursor_data.block_pos.x as f32,
                    block_cursor_data.block_pos.y as f32,
                    block_cursor_data.block_pos.z as f32,
                );
                Transform {
                    translation,
                    ..Transform::IDENTITY
                }
            },
            Color::WHITE,
        );
    }
}
