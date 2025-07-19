use std::{collections::HashMap, fmt::Debug, marker::PhantomData, ops::DerefMut};

use super::chunk_builder::ChunkBuilder;
use crate::chunk_mesh_builder::{builders::BuilderError, ChunkMesh};
use shared::entities::{Chunk, ChunkPos};

#[derive(Debug)]
struct QueuedChunk<T> {
    mesh: ChunkMesh,
    additional_data: T,
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct DelayedData<C> {
    pub build_delay: u32,
    pub custom_data: C,
}

/// A dummy chunk builder that simulates chunk building with a delay.
/// This is useful for testing purposes.
#[derive(Debug)]
pub struct DelayedDummyChunkBuilder<T> {
    _marker: PhantomData<T>,
    queued: HashMap<ChunkPos, QueuedChunk<T>>,
    built_chunks: HashMap<ChunkPos, (ChunkMesh, T)>,
}

impl<T> DelayedDummyChunkBuilder<T> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
            queued: HashMap::new(),
            built_chunks: HashMap::new(),
        }
    }
}

impl<T, C> ChunkBuilder<T> for DelayedDummyChunkBuilder<T>
where
    T: Send + Sync + Default + Debug + DerefMut<Target = DelayedData<C>>,
    C: Send + Sync + Default,
{
    fn force_build(&mut self, chunk_pos: ChunkPos, _chunk: &Chunk, additional_data: T) {
        let mesh = ChunkMesh::default();

        self.queued.insert(
            chunk_pos,
            QueuedChunk {
                mesh,
                additional_data,
            },
        );
    }

    fn try_build(
        &mut self,
        chunk_pos: ChunkPos,
        _chunk: &Chunk,
        additional_data: T,
    ) -> Result<(), BuilderError> {
        if self.queued.contains_key(&chunk_pos) || self.built_chunks.contains_key(&chunk_pos) {
            return Err(BuilderError::ChunkAlreadyExists(chunk_pos));
        }

        let mesh = ChunkMesh::default();

        self.queued.insert(
            chunk_pos,
            QueuedChunk {
                mesh,
                additional_data,
            },
        );

        Ok(())
    }

    fn update(&mut self, _player_pos: ChunkPos) {
        for (chunk_pos, chunk) in self.queued.iter_mut() {
            if chunk.additional_data.build_delay > 0 {
                chunk.additional_data.build_delay -= 1;
            }

            if chunk.additional_data.build_delay == 0 {
                self.built_chunks.insert(
                    *chunk_pos,
                    (
                        std::mem::take(&mut chunk.mesh),
                        std::mem::take(&mut chunk.additional_data),
                    ),
                );
            }
        }

        self.queued
            .retain(|_, chunk| chunk.additional_data.build_delay > 0);
    }

    fn remove_chunk(&mut self, chunk_pos: ChunkPos) {
        self.queued.remove(&chunk_pos);
        self.built_chunks.remove(&chunk_pos);
    }

    fn clear_all(&mut self) {
        self.queued.clear();
        self.built_chunks.clear();
    }

    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, T)> {
        std::mem::take(&mut self.built_chunks)
    }
}
