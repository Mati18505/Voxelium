use std::{cmp::min, collections::HashMap};
use bevy::{prelude::*, tasks::{futures_lite::future, AsyncComputeTaskPool, Task}};

use crate::entities::{Chunk, ChunkPos};

pub trait ChunkProvider: Send + Sync {
    fn load_chunk(&mut self, pos: ChunkPos) -> Chunk;
}

pub struct ChunkLoader {
    chunk_provider: Box<dyn ChunkProvider>,
    chunks_to_load: Vec<ChunkPos>,
    tasks: HashMap<ChunkPos, Task<Chunk>>,
    completed: HashMap<ChunkPos, Chunk>,
}

impl ChunkLoader {
    pub fn new(chunk_provider: Box<dyn ChunkProvider>) -> Self {
        ChunkLoader {
            chunk_provider,
            chunks_to_load: Vec::new(),
            tasks: HashMap::new(),
            completed: HashMap::new(),
        }
    }

    pub fn load_chunk(&mut self, pos: ChunkPos) {
        self.chunks_to_load.push(pos);
    }

    pub fn poll_loaded_chunks(&mut self) -> HashMap<ChunkPos, Chunk> {
        std::mem::take(&mut self.completed)
    }

    const MAX_CHUNKS_PER_UPDATE: usize = 64;

    /// Should be called once per frame.
    pub fn update(&mut self, player_pos: ChunkPos) {
        let chunks_to_update = min(Self::MAX_CHUNKS_PER_UPDATE, self.chunks_to_load.len());

        self.move_k_nearest_chunks_to_back(chunks_to_update, player_pos);

        let median = self.chunks_to_load.len() - chunks_to_update;
        let nearest_chunks = &self.chunks_to_load[median..];

        for pos in nearest_chunks {
            let task = self.chunk_provider.load_chunk(*pos);

            self.completed.insert(*pos, task);
        }

        self.chunks_to_load.truncate(median);

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

    fn move_k_nearest_chunks_to_back(&mut self, k: usize, player_pos: ChunkPos) {
        if self.chunks_to_load.is_empty() {
            return
        }

        let index = self.chunks_to_load.len().saturating_sub(k);

        self.chunks_to_load.select_nth_unstable_by_key(index, |chunk_pos| {
            let distance = Self::chunk_pos_distance_sq(player_pos, *chunk_pos);

            -distance
        });
    }

    fn chunk_pos_distance_sq(a: ChunkPos, b: ChunkPos) -> isize {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let dz = a.z - b.z;

        dx*dx + dy*dy + dz*dz
    }
}