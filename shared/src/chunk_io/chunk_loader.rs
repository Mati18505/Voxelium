use bevy::{
    prelude::*,
    tasks::{futures_lite::future, Task},
};
use std::{collections::HashMap, fmt};

use crate::{
    chunk_io::pending_chunk_queue::PendingChunkQueue,
    entities::{Chunk, ChunkPos},
};

pub trait ChunkProvider: Send + Sync {
    fn load_chunk(&mut self, pos: ChunkPos) -> Chunk;
}

pub struct ChunkLoader {
    chunk_provider: Box<dyn ChunkProvider>,
    chunks_to_load: PendingChunkQueue,
    #[allow(dead_code)]
    tasks: HashMap<ChunkPos, Task<Chunk>>,
    completed: HashMap<ChunkPos, Chunk>,
}

impl ChunkLoader {
    pub fn new(chunk_provider: Box<dyn ChunkProvider>) -> Self {
        ChunkLoader {
            chunk_provider,
            chunks_to_load: PendingChunkQueue::new(),
            tasks: HashMap::new(),
            completed: HashMap::new(),
        }
    }

    pub fn load_chunk(&mut self, pos: ChunkPos) {
        self.chunks_to_load.add_chunk(pos);
    }

    /// Removes chunk from the `chunks_to_load` queue.
    /// If the chunk is currently loading, load operation is cancelled.
    /// If the chunk was already loaded, removes it.
    pub fn cancel_loading_chunk(&mut self, pos: ChunkPos) {
        self.chunks_to_load.remove_chunk(pos);
        self.completed.remove(&pos);
    }

    pub fn get_loaded_chunks(&self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.completed.iter().map(|(chunk_pos, _)| *chunk_pos)
    }

    pub fn poll_loaded_chunks(&mut self) -> HashMap<ChunkPos, Chunk> {
        std::mem::take(&mut self.completed)
    }

    const MAX_CHUNKS_PER_UPDATE: usize = 16;

    /// Should be called once per frame.
    pub fn update(&mut self, player_pos: ChunkPos) {
        let nearest_chunks = self
            .chunks_to_load
            .take_nearest_chunks(Self::MAX_CHUNKS_PER_UPDATE, player_pos);

        for pos in nearest_chunks {
            let task = self.chunk_provider.load_chunk(pos);

            self.completed.insert(pos, task);
        }

        // let completed_tasks = self.poll_completed_tasks();
        // self.completed.extend(completed_tasks);
    }

    #[allow(dead_code)]
    fn poll_completed_tasks(&mut self) -> HashMap<ChunkPos, Chunk> {
        let mut completed: HashMap<ChunkPos, Chunk> = HashMap::default();

        for (chunk_pos, mut task) in self.tasks.iter_mut() {
            if let Some(chunk) = future::block_on(future::poll_once(&mut task)) {
                completed.insert(*chunk_pos, chunk);
            }
        }

        for chunk_pos in completed.keys() {
            self.tasks.remove(chunk_pos);
        }

        completed
    }
}

impl fmt::Debug for ChunkLoader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChunkLoader")
            .field("chunks_to_load", &self.chunks_to_load)
            .field("completed", &self.completed.len())
            .finish()
    }
}
