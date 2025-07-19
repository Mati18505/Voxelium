use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use shared::entities::{Chunk, ChunkPos};

use crate::chunk_mesh_builder::{builders::BuilderError, ChunkMesh};

use super::chunk_builder::ChunkBuilder;

/// A dummy chunk builder that simply simulates chunk building without any delay.
/// This is useful for testing purposes.
#[derive(Debug)]
pub struct DummyChunkBuilder<T> {
    _marker: PhantomData<T>,
    built_chunks: HashMap<ChunkPos, (ChunkMesh, T)>,
}

impl<T> DummyChunkBuilder<T> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            built_chunks: HashMap::new(),
        }
    }
}

impl<T: Send + Sync + Default + Debug> ChunkBuilder<T> for DummyChunkBuilder<T> {
    fn force_build(&mut self, chunk_pos: ChunkPos, _chunk: &Chunk, additional_data: T) {
        self.built_chunks
            .insert(chunk_pos, (ChunkMesh::default(), additional_data));
    }

    fn try_build(
        &mut self,
        chunk_pos: ChunkPos,
        _chunk: &Chunk,
        additional_data: T,
    ) -> Result<(), BuilderError> {
        if self.built_chunks.contains_key(&chunk_pos) {
            return Err(BuilderError::ChunkAlreadyExists(chunk_pos));
        }

        self.built_chunks
            .insert(chunk_pos, (ChunkMesh::default(), additional_data));
        Ok(())
    }

    fn update(&mut self, _player_pos: ChunkPos) {
        // No operation
    }

    fn remove_chunk(&mut self, chunk_pos: ChunkPos) {
        self.built_chunks.remove(&chunk_pos);
    }

    fn clear_all(&mut self) {
        self.built_chunks.clear();
    }

    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, T)> {
        std::mem::take(&mut self.built_chunks)
    }
}
