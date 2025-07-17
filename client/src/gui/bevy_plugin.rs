use bevy::prelude::*;
use shared::entities::*;

use crate::{bevy_types::AppStates, orchestrator};

pub struct GUIPlugin;
impl Plugin for GUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gui).add_systems(
            Update,
            update_block_cursor.run_if(in_state(AppStates::InGame)),
        );
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

fn update_block_cursor(
    mut gizmos: Gizmos,
    mut looked_at_block_change_ev: EventReader<orchestrator::LookedAtBlockChangedEvent>,
    mut q_block_cursor_data: Query<&mut BlockCursorData>,
) {
    let mut block_cursor_data = match q_block_cursor_data.single_mut() {
        Ok(block_cursor_data) => block_cursor_data,
        Err(_) => {
            warn!("Gizmo data not found for update_gizmo!");
            return;
        }
    };

    for ev in looked_at_block_change_ev.read() {
        block_cursor_data.visible = ev.block_type.affect_raycast;
        block_cursor_data.block_pos = ev.block_pos;
    }

    if block_cursor_data.visible {
        gizmos.cuboid(
            {
                let translation = Vec3::new(
                    block_cursor_data.block_pos.x as f32,
                    block_cursor_data.block_pos.z as f32,
                    -block_cursor_data.block_pos.y as f32,
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
