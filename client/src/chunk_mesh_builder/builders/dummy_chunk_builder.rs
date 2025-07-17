use std::collections::HashMap;

use shared::entities::{Chunk, ChunkPos};

use crate::chunk_mesh_builder::ChunkMesh;

use super::chunk_builder::ChunkBuilder;

pub struct DummyChunkBuilder;

impl DummyChunkBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl ChunkBuilder<()> for DummyChunkBuilder {
    fn build_chunk(&mut self, _chunk_pos: ChunkPos, _chunk: &Chunk, _additional_data: Option<()>) {
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

    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, Option<()>)> {
        HashMap::new()
    }
}
