use bevy::prelude::*;

use super::chunk_manager;
use super::chunk_manager::WorldChunkUpdate;

#[derive(Event, Debug, Clone, PartialEq)]
pub struct WorldChunkUpdateEvent {
    pub chunk_update: WorldChunkUpdate,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EventManager {
    chunk_update_events: Vec<WorldChunkUpdateEvent>,
}

impl chunk_manager::EventCallback for EventManager {
    fn chunk_update_callback(&mut self, chunk_update: WorldChunkUpdate) {
        let ev = WorldChunkUpdateEvent { chunk_update };

        self.chunk_update_events.push(ev);
    }
}

impl EventManager {
    pub fn process_pending(
        &mut self,
        mut events: EventWriter<WorldChunkUpdateEvent>,
    ) {
        for ev in std::mem::take(&mut self.chunk_update_events) {
            events.write(ev);
        }

        assert!(self.chunk_update_events.len() == 0);
    }
}