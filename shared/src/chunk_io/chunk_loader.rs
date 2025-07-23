use bevy::{
    prelude::*,
    tasks::{futures_lite::future, Task},
};
use std::{
    collections::HashMap,
    fmt::{self, Debug},
};

use crate::{
    chunk_io::pending_chunk_queue::PendingChunkQueue,
    entities::{Chunk, ChunkPos},
};

pub trait ChunkProvider: Send + Sync + Debug {
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

    /// Replaces the current chunk provider.
    /// Only new chunks will be loaded using the new provider.
    /// Previously loaded chunks are not affected.
    pub fn change_chunk_provider(&mut self, chunk_provider: Box<dyn ChunkProvider>) {
        self.chunk_provider = chunk_provider;
    }

    /// Clears all currently loaded chunks and adds them to `chunks_to_load` queue.
    /// This is useful when changing the chunk provider.
    pub fn reload_all(&mut self) {
        for chunk_pos in self.completed.keys() {
            self.chunks_to_load.add_chunk(*chunk_pos);
        }
        self.completed.clear();
    }

    /// Removes chunk from the `chunks_to_load` queue.
    /// If the chunk is currently loading, load operation is cancelled.
    /// If the chunk was already loaded, removes it.
    pub fn cancel_loading_chunk(&mut self, pos: ChunkPos) {
        self.chunks_to_load.remove_chunk(pos);
        self.completed.remove(&pos);
    }

    /// Cancels loading of all chunks and clears loaded ones.
    pub fn clear_all(&mut self) {
        self.chunks_to_load = PendingChunkQueue::default();
        self.completed.clear();
    }

    pub fn poll_loaded_chunks(&mut self) -> HashMap<ChunkPos, Chunk> {
        std::mem::take(&mut self.completed)
    }

    const MAX_CHUNKS_PER_UPDATE: usize = 16;

    /// Should be called once per frame.
    pub fn update(&mut self, player_pos: ChunkPos) {
        let my_span = info_span!("update_chunk_loader", name = "update_chunk_loader").entered();

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

impl Debug for ChunkLoader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChunkLoader")
            .field("chunks_to_load", &self.chunks_to_load)
            .field("completed", &self.completed.len())
            .field("provider", &self.chunk_provider)
            .finish()
    }
}
