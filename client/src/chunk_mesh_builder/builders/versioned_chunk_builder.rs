use std::collections::HashMap;

use super::dummy_chunk_builder::DummyChunkBuilder;
use shared::entities::{Chunk, ChunkPos};

use crate::chunk_mesh_builder::{builders::ChunkBuilder, ChunkMesh};

use super::chunk_builder::Versioned;

pub struct VersionedChunkBuilder {
    chunk_builder: Box<dyn ChunkBuilder<()>>,
    chunk_mesh_versions: HashMap<ChunkPos, Version>,
}

type Version = u64;

impl VersionedChunkBuilder {
    /// Creates a new versioned chunk builder with the provided chunk builder.
    /// Newest call to this function equals the latest version of the chunk.
    pub fn new(chunk_builder: Box<dyn ChunkBuilder<()>>) -> Self {
        Self {
            chunk_builder,
            chunk_mesh_versions: HashMap::new(),
        }
    }

    fn get_chunk_mesh_version(&self, pos: ChunkPos) -> Version {
        self.chunk_mesh_versions.get(&pos).copied().unwrap_or(0)
    }

    fn increment_chunk_mesh_version(&mut self, pos: ChunkPos) -> Version {
        let incremented_version = *self
            .chunk_mesh_versions
            .entry(pos)
            .and_modify(|e| *e = e.wrapping_add(1))
            .or_insert(1);
        Version::from(incremented_version)
    }
}

impl ChunkBuilder<()> for VersionedChunkBuilder {
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, _additional_data: Option<()>) {
        self.chunk_builder.build_chunk(chunk_pos, chunk, None);
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

    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, Option<()>)> {
        self.chunk_builder.poll_completed()
    }
}

impl Versioned for VersionedChunkBuilder {
    fn is_chunk_with_latest_version_built(&self, chunk_pos: ChunkPos) -> bool {
        unimplemented!()
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
        let mut builder = VersionedChunkBuilder::new(Box::new(DummyChunkBuilder::new()));
        let chunk_pos = ChunkPos::new(0, 0, 0);
        let chunk = Chunk::default();

        // Initial version should be 0.
        builder.build_chunk(chunk_pos, &chunk, None);
        let version = builder.get_chunk_mesh_version(chunk_pos);
        assert_eq!(version, 0);

        // Check if version increments correctly.
        builder.increment_chunk_mesh_version(chunk_pos);
        let version = builder.get_chunk_mesh_version(chunk_pos);
        assert_eq!(version, 1);

        // Check if it wraps around correctly.
        builder.chunk_mesh_versions.insert(chunk_pos, Version::MAX);

        let version = builder.get_chunk_mesh_version(chunk_pos);
        assert_eq!(version, Version::MAX);

        builder.increment_chunk_mesh_version(chunk_pos);

        let version = builder.get_chunk_mesh_version(chunk_pos);
        assert_eq!(version, 0);
    }
}
