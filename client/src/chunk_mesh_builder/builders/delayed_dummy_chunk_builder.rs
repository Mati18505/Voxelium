use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::chunk_builder::ChunkBuilder;
use crate::chunk_mesh_builder::ChunkMesh;
use shared::entities::{Chunk, ChunkPos};

struct QueuedChunk<T> {
    mesh: ChunkMesh,
    additional_data: T,
}

#[derive(Default)]
pub struct DelayedData<C> {
    pub build_delay: u32,
    pub custom_data: C,
}

/// A dummy chunk builder that simulates chunk building with a delay.
/// This is useful for testing purposes.
pub struct DelayedDummyChunkBuilder<T> {
    _marker: PhantomData<T>,
    queued: Vec<(ChunkPos, QueuedChunk<T>)>,
    built_chunks: Vec<(ChunkPos, (ChunkMesh, T))>,
}

impl<T> DelayedDummyChunkBuilder<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            queued: Vec::new(),
            built_chunks: Vec::new(),
        }
    }
}

impl<T, C> ChunkBuilder<T> for DelayedDummyChunkBuilder<T>
where
    T: Send + Sync + Default + DerefMut<Target = DelayedData<C>>,
    C: Send + Sync + Default,
{
    fn build_chunk(&mut self, chunk_pos: ChunkPos, _chunk: &Chunk, additional_data: T) {
        let mesh = ChunkMesh::default();

        self.queued.push((
            chunk_pos,
            QueuedChunk {
                mesh,
                additional_data,
            },
        ));
    }

    fn update(&mut self, _player_pos: ChunkPos) {
        for (chunk_pos, chunk) in self.queued.iter_mut() {
            if chunk.additional_data.build_delay > 0 {
                chunk.additional_data.build_delay -= 1;
            }

            if chunk.additional_data.build_delay == 0 {
                self.built_chunks.push((
                    *chunk_pos,
                    (
                        std::mem::take(&mut chunk.mesh),
                        std::mem::take(&mut chunk.additional_data),
                    ),
                ));
            }
        }

        self.queued
            .retain(|(_, chunk)| chunk.additional_data.build_delay > 0);
    }

    fn remove_chunk(&mut self, chunk_pos: ChunkPos) {
        self.queued.retain(|(pos, _)| *pos != chunk_pos);
        self.built_chunks.retain(|(pos, _)| *pos != chunk_pos);
    }

    fn clear_all(&mut self) {
        self.queued.clear();
        self.built_chunks.clear();
    }

    fn poll_completed(&mut self) -> Vec<(ChunkPos, (ChunkMesh, T))> {
        std::mem::take(&mut self.built_chunks)
    }
}
