use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use shared::entities::{Chunk, ChunkPos};

use crate::chunk_mesh_builder::ChunkMesh;

use super::chunk_builder::ChunkBuilder;

/// A dummy chunk builder that simply simulates chunk building without any delay.
/// This is useful for testing purposes.
#[derive(Debug)]
pub struct DummyChunkBuilder<T> {
    _marker: PhantomData<T>,
    built_chunks: Vec<(ChunkPos, (ChunkMesh, T))>,
}

impl<T> DummyChunkBuilder<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            built_chunks: Vec::new(),
        }
    }
}

impl<T: Send + Sync + Default + Debug> ChunkBuilder<T> for DummyChunkBuilder<T> {
    fn build_chunk(&mut self, chunk_pos: ChunkPos, _chunk: &Chunk, additional_data: T) {
        self.built_chunks
            .push((chunk_pos, (ChunkMesh::default(), additional_data)));
    }

    fn update(&mut self, _player_pos: ChunkPos) {
        // No operation
    }

    fn remove_chunk(&mut self, chunk_pos: ChunkPos) {
        self.built_chunks.retain(|(pos, _)| *pos != chunk_pos);
    }

    fn clear_all(&mut self) {
        self.built_chunks.clear();
    }

    fn poll_completed(&mut self) -> Vec<(ChunkPos, (ChunkMesh, T))> {
        std::mem::take(&mut self.built_chunks)
    }
}
