use std::{collections::HashMap, marker::PhantomData};

use shared::entities::{Chunk, ChunkPos};

use crate::chunk_mesh_builder::ChunkMesh;

use super::chunk_builder::ChunkBuilder;

pub struct DummyChunkBuilder<T> {
    _marker: PhantomData<T>,
}

impl<T> DummyChunkBuilder<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: Send + Sync> ChunkBuilder<T> for DummyChunkBuilder<T> {
    fn build_chunk(&mut self, _chunk_pos: ChunkPos, _chunk: &Chunk, _additional_data: Option<T>) {
        // No operation
    }

    fn update(&mut self, _player_pos: ChunkPos) {
        // No operation
    }

    fn remove_chunk(&mut self, _chunk_pos: ChunkPos) {
        // No operation
    }

    fn clear_all(&mut self) {
        // No operation
    }

    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, Option<T>)> {
        HashMap::new()
    }
}
