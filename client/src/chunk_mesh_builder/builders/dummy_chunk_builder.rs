use std::{collections::HashMap, marker::PhantomData};

use shared::entities::{Chunk, ChunkPos};

use crate::chunk_mesh_builder::ChunkMesh;

use super::chunk_builder::ChunkBuilder;

/// A dummy chunk builder that simply simulates chunk building without any delay.
/// This is useful for testing purposes.
pub struct DummyChunkBuilder<T> {
    _marker: PhantomData<T>,
    builded_chunks: HashMap<ChunkPos, (ChunkMesh, T)>,
}

impl<T> DummyChunkBuilder<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            builded_chunks: HashMap::new(),
        }
    }
}

impl<T: Send + Sync + Default> ChunkBuilder<T> for DummyChunkBuilder<T> {
    fn build_chunk(&mut self, chunk_pos: ChunkPos, _chunk: &Chunk, additional_data: T) {
        self.builded_chunks
            .insert(chunk_pos, (ChunkMesh::default(), additional_data));
    }

    fn update(&mut self, _player_pos: ChunkPos) {
        // No operation
    }

    fn remove_chunk(&mut self, chunk_pos: ChunkPos) {
        self.builded_chunks.remove(&chunk_pos);
    }

    fn clear_all(&mut self) {
        self.builded_chunks.clear();
    }

    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, T)> {
        std::mem::take(&mut self.builded_chunks)
    }
}
