use crate::chunk_mesh_builder::{ChunkMesh, VoxelMesher};
use shared::entities::{Chunk, ChunkPos};
use std::{collections::HashMap, fmt::Debug};

pub trait ChunkBuilder<T: Send + Sync + Default>: Debug {
    /// Adds a chunk to the builder.
    /// Additional data can be used to store version or other metadata.
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, additional_data: T);

    /// Updates the builder state based on the player's position.
    /// Should be called once per frame.
    /// Can optimize chunk building based on player position.
    fn update(&mut self, player_pos: ChunkPos);

    /// Removes a chunk from the builder, cancelling build operation.
    fn remove_chunk(&mut self, chunk_pos: ChunkPos);

    /// Removes all chunks from the builder, cancelling all build operations.
    fn clear_all(&mut self);

    /// Returns a vector of all built chunks and additional data.
    /// Returned chunks are removed from the builder.
    fn poll_completed(&mut self) -> Vec<(ChunkPos, (ChunkMesh, T))>;
}

pub trait Versioned<T: Send + Sync + Default> {
    /// Returns true if a chunk is built with the latest version.
    fn is_chunk_with_latest_version_built(&self, chunk_pos: ChunkPos) -> bool;

    /// Returns a chunk built with the latest version.
    /// If latest version is different than version of the chunk, or no chunk is built, it will return None.
    /// Returned chunk is removed from the builder.
    fn take_chunk_built_with_latest_version(
        &mut self,
        chunk_pos: ChunkPos,
    ) -> Option<(ChunkMesh, T)>;
}
