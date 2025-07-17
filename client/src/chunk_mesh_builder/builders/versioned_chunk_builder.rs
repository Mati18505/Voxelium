use std::collections::HashMap;

use super::dummy_chunk_builder::DummyChunkBuilder;
use shared::entities::{Chunk, ChunkPos};

use crate::chunk_mesh_builder::{builders::ChunkBuilder, ChunkMesh};

use super::chunk_builder::Versioned;

pub struct VersionedChunkBuilder<T: Send + Sync> {
    /// Internal chunk builder.
    chunk_builder: Box<dyn ChunkBuilder<DecoratedData<T>>>,

    /// Stores latest version for each chunk.
    latest_chunk_mesh_versions: HashMap<ChunkPos, Version>,

    /// Stores only chunks that are built with the latest version.
    /// If chunk version is updated, the chunk will be removed from this map.
    latest_builded_chunks: HashMap<ChunkPos, (ChunkMesh, Option<T>)>,
}

type Version = u64;

struct DecoratedData<T: Send + Sync> {
    pub data: Option<T>,
    version: Version,
}

impl<T: Send + Sync> VersionedChunkBuilder<T> {
    /// Creates a new versioned chunk builder with the provided chunk builder.
    /// Newest call to this function equals the latest version of the chunk.
    pub fn new(chunk_builder: Box<dyn ChunkBuilder<DecoratedData<T>>>) -> Self {
        Self {
            chunk_builder,
            latest_chunk_mesh_versions: HashMap::new(),
            latest_builded_chunks: HashMap::new(),
        }
    }

    fn get_chunk_mesh_version(&self, pos: ChunkPos) -> Version {
        self.latest_chunk_mesh_versions
            .get(&pos)
            .copied()
            .unwrap_or(0)
    }

    fn increment_chunk_mesh_version(&mut self, pos: ChunkPos) -> Version {
        let incremented_version = *self
            .latest_chunk_mesh_versions
            .entry(pos)
            .and_modify(|e| *e = e.wrapping_add(1))
            .or_insert(1);
        Version::from(incremented_version)
    }
}

impl<T: Send + Sync> ChunkBuilder<T> for VersionedChunkBuilder<T> {
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, additional_data: Option<T>) {
        let version = self.increment_chunk_mesh_version(chunk_pos);
        let decorated_data = DecoratedData {
            data: additional_data,
            version,
        };

        self.chunk_builder
            .build_chunk(chunk_pos, chunk, Some(decorated_data));
    }

    fn update(&mut self, player_pos: ChunkPos) {
        self.chunk_builder.update(player_pos);
    }

    fn remove_chunk(&mut self, chunk_pos: ChunkPos) {
        self.chunk_builder.remove_chunk(chunk_pos);
    }

    fn clear_all(&mut self) {
        self.chunk_builder.clear_all();
    }

    /// Returns only chunks that are built with the latest version.
    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, Option<T>)> {
        std::mem::take(&mut self.latest_builded_chunks)
    }
}

impl<T: Send + Sync> Versioned for VersionedChunkBuilder<T> {
    fn is_chunk_with_latest_version_built(&self, chunk_pos: ChunkPos) -> bool {
        let version = self.get_chunk_mesh_version(chunk_pos);
        self.latest_builded_chunks.contains_key(&chunk_pos)
    }

    fn take_chunk_built_with_latest_version(&mut self, chunk_pos: ChunkPos) -> Option<ChunkMesh> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::entities::{Chunk, ChunkPos};

    #[test]
    fn test_chunk_mesh_version() {
        let inner_builder = Box::new(DummyChunkBuilder::new());
        let mut builder = VersionedChunkBuilder::<()>::new(inner_builder);
        let chunk_pos = ChunkPos::new(0, 0, 0);
        let chunk = Chunk::default();

        // Initial version should be 0.
        let version = builder.get_chunk_mesh_version(chunk_pos);
        assert_eq!(version, 0);

        // Check if version increments correctly.
        builder.increment_chunk_mesh_version(chunk_pos);
        let version = builder.get_chunk_mesh_version(chunk_pos);
        assert_eq!(version, 1);

        // Check if it wraps around correctly.
        builder
            .latest_chunk_mesh_versions
            .insert(chunk_pos, Version::MAX);

        let version = builder.get_chunk_mesh_version(chunk_pos);
        assert_eq!(version, Version::MAX);

        builder.increment_chunk_mesh_version(chunk_pos);

        let version = builder.get_chunk_mesh_version(chunk_pos);
        assert_eq!(version, 0);
    }
}
