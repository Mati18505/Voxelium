use bevy::{log, prelude::*};

use crate::{
    bevy_types::AppStates,
    chunk_manager::{chunk_state, events::*, resources::*},
};
use shared::entities::{ChunkPos, ChunkRepository};

pub struct ChunkStateManagerPlugin;
impl Plugin for ChunkStateManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ChunkStateTransition>().add_systems(
            Update,
            process_state_update_requests.run_if(in_state(AppStates::InGame)),
        );
    }
}

fn process_state_update_requests(
    mut state_update_req_ev: EventReader<StateUpdateRequest>,
    mut chunk_state_transition_ev: EventWriter<ChunkStateTransition>,
) {
    for ev in state_update_req_ev.read() {
        let chunk_pos = ev.chunk_pos;
        let prev_state = ev.curr_state;
        let next_state = chunk_state::get_next_chunk_state(prev_state, ev.chunk_status);
        let transition = chunk_state::get_chunk_transition(prev_state, next_state);

        if let Some(transition) = transition {
            trace!("Chunk transition in chunk {chunk_pos:?}: {prev_state:?} -> {next_state:?}");

            chunk_state_transition_ev.write(ChunkStateTransition {
                chunk_pos,
                transition,
                new_state: next_state,
            });
        } 
    }
}
