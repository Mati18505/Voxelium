use bevy::prelude::*;

use super::chunk_state_manager::WorldChunkUpdate;

/// Bevy event.
#[derive(Event, Debug, Clone, PartialEq)]
pub struct WorldChunkUpdateEvent {
    pub chunk_update: WorldChunkUpdate,
}

#[derive(Debug, Clone)]
pub struct EventManager {
    chunk_update_rx: crossbeam_channel::Receiver<WorldChunkUpdate>,
}

impl EventManager {
    pub fn new(chunk_update_rx: crossbeam_channel::Receiver<WorldChunkUpdate>) -> Self {
        Self {
            chunk_update_rx,
        }
    }
    pub fn process_pending(
        &mut self,
        mut events: EventWriter<WorldChunkUpdateEvent>,
    ) {
        for ev in self.chunk_update_rx.try_iter() {
            events.write(WorldChunkUpdateEvent { chunk_update: ev });
        }
    }
}