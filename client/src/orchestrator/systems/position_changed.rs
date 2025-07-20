use bevy::prelude::*;
use shared::entities::{BlockPos, ChunkPos};

use crate::{controller, orchestrator::ChunkPosChangedEvent};

pub struct PositionChangeEventPlugin;

impl Plugin for PositionChangeEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChunkPosChangedEvent>()
            .add_systems(Startup, initialize_chunk_pos_data)
            .add_systems(
                Update,
                (
                    emit_chunk_change_events.after(update_chunk_pos_data),
                ),
            );
    }
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ChunkPositionData {
    last_chunk_pos: ChunkPos,
}

pub fn initialize_chunk_pos_data(mut commands: Commands) {
    commands.spawn(ChunkPositionData::default());
}

pub fn update_chunk_pos_data(
    controller_pos_changed_ev: EventReader<controller::PositionChangeEvent>,
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
        let block_pos = BlockPos::new(ev.new_pos.x, ev.new_pos.y, ev.new_pos.z);
        let chunk_pos = ChunkPos::from(block_pos);

        if chunk_pos != chunk_pos_data.last_chunk_pos {
            chunk_pos_data.last_chunk_pos = chunk_pos;
        }
    }
    println!("1");
}

pub fn emit_chunk_change_events(
    q_chunk_pos_data: Query<&ChunkPositionData, Changed<ChunkPositionData>>,
    chunk_pos_changed_ev: EventWriter<ChunkPosChangedEvent>,
) {
    for chunk_pos_data in &q_chunk_pos_data {
        chunk_pos_changed_ev.send(ChunkPosChangedEvent {
            chunk_pos: chunk_pos_data.last_chunk_pos,
        });
    }
    println!("2");
}
