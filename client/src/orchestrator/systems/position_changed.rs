use bevy::prelude::*;
use shared::entities::{BlockPos, ChunkPos};

use crate::{
    bevy_types::AppStates,
    controller::{self, PositionChangeEvent},
    orchestrator::ChunkPosChangedEvent,
};

pub struct PositionChangeEventPlugin;

impl Plugin for PositionChangeEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChunkPosChangedEvent>()
            .add_systems(Startup, initialize_chunk_pos_data)
            .add_systems(
                Update,
                (
                    update_chunk_pos_data,
                    emit_chunk_change_events.after(update_chunk_pos_data),
                ),
            );
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ChunkPositionData {
    last_chunk_pos: Option<ChunkPos>,
}

pub fn initialize_chunk_pos_data(mut commands: Commands) {
    commands.spawn(ChunkPositionData::default());
}

pub fn update_chunk_pos_data(
    mut controller_pos_changed_ev: EventReader<PositionChangeEvent>,
    mut q_chunk_pos_data: Query<&mut ChunkPositionData>,
) {
    let mut chunk_pos_data = match q_chunk_pos_data.single_mut() {
        Ok(data) => data,
        Err(_) => {
            warn!("ChunkPosData not found for 'emit_chunk_change_events'!");
            return;
        }
    };

    for ev in controller_pos_changed_ev.read() {
        // Convert bevy direction to our direction
        let block_pos = BlockPos::new(
            ev.new_pos.x as isize,
            -ev.new_pos.z as isize,
            ev.new_pos.y as isize,
        );
        let chunk_pos = ChunkPos::from(block_pos);

        if let Some(last_chunk_pos) = chunk_pos_data.last_chunk_pos {
            if chunk_pos != last_chunk_pos {
                chunk_pos_data.last_chunk_pos = Some(chunk_pos);
            }
        } else {
            chunk_pos_data.last_chunk_pos = Some(chunk_pos);
        }
    }
}

pub fn emit_chunk_change_events(
    q_chunk_pos_data: Query<&ChunkPositionData, Changed<ChunkPositionData>>,
    mut chunk_pos_changed_ev: EventWriter<ChunkPosChangedEvent>,
) {
    for chunk_pos_data in &q_chunk_pos_data {
        if let Some(chunk_pos) = chunk_pos_data.last_chunk_pos {
            chunk_pos_changed_ev.write(ChunkPosChangedEvent {
                chunk_pos: chunk_pos,
            });
        }
    }
}
