use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use super::chunk_builder::Versioned;
use super::dummy_chunk_builder::DummyChunkBuilder;
use crate::chunk_mesh_builder::{builders::ChunkBuilder, ChunkMesh};
use shared::entities::{Chunk, ChunkPos};

pub struct VersionedChunkBuilder<T: Send + Sync + Default> {
    /// Internal chunk builder.
    chunk_builder: Box<dyn ChunkBuilder<DecoratedData<T>>>,

    /// Stores latest version for each chunk.
    latest_chunk_mesh_versions: HashMap<ChunkPos, Version>,

    /// Stores only chunks that are built with the latest version.
    /// If chunk version is updated, the chunk will be removed from this map.
    latest_builded_chunks: HashMap<ChunkPos, (ChunkMesh, DecoratedData<T>)>,
}

type Version = u64;

#[derive(Debug, Clone, Default)]
struct DecoratedData<T: Send + Sync + Default> {
    pub data: T,
    version: Version,
}

impl<T: Send + Sync + Default> Deref for DecoratedData<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T: Send + Sync + Default> DerefMut for DecoratedData<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: Send + Sync + Default> VersionedChunkBuilder<T> {
    /// Creates a new versioned chunk builder with the provided chunk builder.
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

impl<T: Send + Sync + Default> ChunkBuilder<T> for VersionedChunkBuilder<T> {
    /// Newest call to this function equals the latest version of the chunk.
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, additional_data: T) {
        let version = self.increment_chunk_mesh_version(chunk_pos);
        let decorated_data = DecoratedData {
            data: additional_data,
            version,
        };

        self.chunk_builder
            .build_chunk(chunk_pos, chunk, decorated_data);
    }

    fn update(&mut self, player_pos: ChunkPos) {
        self.chunk_builder.update(player_pos);

        let completed: HashMap<ChunkPos, (ChunkMesh, DecoratedData<T>)> = self
            .chunk_builder
            .poll_completed()
            .into_iter()
            .filter(|(chunk_pos, (_, decorated_data))| {
                let latest_chunk_mesh_version = self.get_chunk_mesh_version(*chunk_pos);

                decorated_data.version == latest_chunk_mesh_version
            })
            .collect();

        let dbg: Vec<&ChunkPos> = completed.iter().map(|(chunk_pos, _)| chunk_pos).collect();

        self.latest_builded_chunks.extend(completed);
    }

    fn remove_chunk(&mut self, chunk_pos: ChunkPos) {
        self.chunk_builder.remove_chunk(chunk_pos);
    }

    fn clear_all(&mut self) {
        self.chunk_builder.clear_all();
    }

    /// Returns only chunks that are built with the latest version.
    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, T)> {
        // Convert DecoratedData to T.
        std::mem::take(&mut self.latest_builded_chunks)
            .into_iter()
            .map(|(pos, (mesh, data))| (pos, (mesh, data.data)))
            .collect()
    }
}

impl<T: Send + Sync + Default> Versioned<T> for VersionedChunkBuilder<T> {
    fn is_chunk_with_latest_version_built(&self, chunk_pos: ChunkPos) -> bool {
        let version = self.get_chunk_mesh_version(chunk_pos);

        if let Some(chunk) = self.latest_builded_chunks.get(&chunk_pos) {
            chunk.1.version == version
        } else {
            false
        }
    }

    fn take_chunk_built_with_latest_version(
        &mut self,
        chunk_pos: ChunkPos,
    ) -> Option<(ChunkMesh, T)> {
        self.latest_builded_chunks
            .remove(&chunk_pos)
            .map(|(mesh, data)| (mesh, data.data))
    }
}

#[cfg(test)]
mod tests {
    use crate::chunk_mesh_builder::builders::delayed_dummy_chunk_builder::{
        self, DelayedData, DelayedDummyChunkBuilder,
    };

    use super::*;
    use rand::Rng;
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

    #[test]
    fn test_build_chunk() {
        let inner_builder = Box::new(DummyChunkBuilder::new());
        let mut builder = VersionedChunkBuilder::<u32>::new(inner_builder);
        let chunk_pos = ChunkPos::new(0, 0, 0);
        let chunk = Chunk::default();

        builder.build_chunk(chunk_pos, &chunk, 42);
        builder.update(ChunkPos::new(0, 0, 0));

        assert!(builder.is_chunk_with_latest_version_built(chunk_pos));

        let builded_chunk = builder.take_chunk_built_with_latest_version(chunk_pos);
        assert!(builded_chunk.is_some());

        let (_chunk_mesh, additional_data) = builded_chunk.unwrap();
        assert_eq!(additional_data, 42);
    }

    #[test]
    fn test_chunk_always_has_newest_version() {
        let inner_builder = Box::new(DelayedDummyChunkBuilder::new());
        let mut builder = VersionedChunkBuilder::<DelayedData<i32>>::new(inner_builder);
        let chunk_pos = ChunkPos::new(0, 0, 0);
        let player_pos = ChunkPos::new(0, 0, 0);
        let chunk = Chunk::default();
        let mut newest_data = 0;

        // Build initial chunks with different data.
        builder.build_chunk(
            chunk_pos,
            &chunk,
            DelayedData {
                build_delay: 0,
                custom_data: -20,
            },
        );
        builder.build_chunk(
            chunk_pos,
            &chunk,
            DelayedData {
                build_delay: 20,
                custom_data: -10,
            },
        );
        builder.build_chunk(
            chunk_pos,
            &chunk,
            DelayedData {
                build_delay: 10,
                custom_data: newest_data,
            },
        );

        for frame in 0..50 {
            // Every 4th frame, we build a new chunk with the newest data.
            if frame % 4 == 0 {
                newest_data += 1;
                builder.build_chunk(
                    chunk_pos,
                    &chunk,
                    DelayedData {
                        build_delay: rand::rng().random_range(1..10),
                        custom_data: newest_data,
                    },
                );
            }

            builder.update(player_pos);

            for (chunk_pos, (chunk_mesh, value)) in builder.poll_completed() {
                // Check if each built chunk returned from poll_completed is newest.
                assert_eq!(value.custom_data, newest_data);
                println!("completed {:?}", value.custom_data);
            }
        }
    }

    #[test]
    fn test_only_one_chunk_has_newest_version() {
        let inner_builder = Box::new(DelayedDummyChunkBuilder::new());
        let mut builder = VersionedChunkBuilder::<DelayedData<i32>>::new(inner_builder);
        let chunk_pos = ChunkPos::new(0, 0, 0);
        let player_pos = ChunkPos::new(0, 0, 0);
        let chunk = Chunk::default();
        let mut total_builded = 0;

        // Build initial chunks with different delay and data.
        builder.build_chunk(
            chunk_pos,
            &chunk,
            DelayedData {
                build_delay: 0,
                custom_data: -20,
            },
        );
        builder.build_chunk(
            chunk_pos,
            &chunk,
            DelayedData {
                build_delay: 35,
                custom_data: -10,
            },
        );
        builder.build_chunk(
            chunk_pos,
            &chunk,
            DelayedData {
                build_delay: 1,
                custom_data: 30,
            },
        );

        for frame in 0..50 {
            builder.update(player_pos);

            for (chunk_pos, (chunk_mesh, value)) in builder.poll_completed() {
                // Check if built chunk returned from poll_completed is newest.
                assert_eq!(value.custom_data, 30);
                total_builded += 1;
            }
        }

        assert_eq!(total_builded, 1);
    }
}
