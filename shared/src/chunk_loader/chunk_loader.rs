use std::{cmp::min, collections::HashMap};
use bevy::{prelude::*, tasks::{futures_lite::future, AsyncComputeTaskPool, Task}};

use crate::{chunk_loader::pending_chunk_queue::PendingChunkQueue, entities::{Chunk, ChunkPos}};

pub trait ChunkProvider: Send + Sync {
    fn load_chunk(&mut self, pos: ChunkPos) -> Chunk;
}

pub struct ChunkLoader {
    chunk_provider: Box<dyn ChunkProvider>,
    chunks_to_load: PendingChunkQueue,
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

    pub fn poll_loaded_chunks(&mut self) -> HashMap<ChunkPos, Chunk> {
        std::mem::take(&mut self.completed)
    }

    const MAX_CHUNKS_PER_UPDATE: usize = 16;

    /// Should be called once per frame.
    pub fn update(&mut self, player_pos: ChunkPos) {
        let nearest_chunks = self.chunks_to_load.take_nearest_chunks(Self::MAX_CHUNKS_PER_UPDATE, player_pos);

        for pos in nearest_chunks {
            let task = self.chunk_provider.load_chunk(pos);

            self.completed.insert(pos, task);
        }

        // let completed_tasks = self.poll_completed_tasks();
        // self.completed.extend(completed_tasks);
    }

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