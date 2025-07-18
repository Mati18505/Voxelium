use std::{
    collections::{HashMap, VecDeque},
    marker::PhantomData,
};

use rand::Rng;
use shared::entities::{Chunk, ChunkPos};

use crate::chunk_mesh_builder::ChunkMesh;

use super::chunk_builder::ChunkBuilder;

struct QueuedChunk<T> {
    mesh: ChunkMesh,
    additional_data: T,
    delay: u32,
}

/// A dummy chunk builder that simulates chunk building with a delay.
/// This is useful for testing purposes.
pub struct DelayedDummyChunkBuilder<T> {
    _marker: PhantomData<T>,
    queued: HashMap<ChunkPos, QueuedChunk<T>>,
    builded_chunks: HashMap<ChunkPos, (ChunkMesh, T)>,
}

impl<T> DelayedDummyChunkBuilder<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            queued: HashMap::new(),
            builded_chunks: HashMap::new(),
        }
    }
}

impl<T: Send + Sync + Default> ChunkBuilder<T> for DelayedDummyChunkBuilder<T> {
    fn build_chunk(&mut self, chunk_pos: ChunkPos, _chunk: &Chunk, additional_data: T) {
        let delay = rand::thread_rng().random_range(0..5);
        let mesh = ChunkMesh::default();

        self.queued.insert(
            chunk_pos,
            QueuedChunk {
                mesh,
                additional_data,
                delay,
            },
        );
    }

    fn update(&mut self, _player_pos: ChunkPos) {
        let mut completed: Vec<ChunkPos> = Vec::default();

        for (chunk_pos, chunk) in self.queued.iter_mut() {
            if chunk.delay <= 0 {
                completed.push(*chunk_pos);
            } else {
                chunk.delay -= 1;
            }
        }

        for chunk_pos in completed {
            let chunk = self.queued.remove(&chunk_pos).unwrap();
            let chunk = (chunk.mesh, chunk.additional_data);

            self.builded_chunks.insert(chunk_pos, chunk);
        }
    }

    fn remove_chunk(&mut self, chunk_pos: ChunkPos) {
        self.queued.remove(&chunk_pos);
        self.builded_chunks.remove(&chunk_pos);
    }

    fn clear_all(&mut self) {
        self.queued.clear();
        self.builded_chunks.clear();
    }

    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, T)> {
        std::mem::take(&mut self.builded_chunks)
    }
}
