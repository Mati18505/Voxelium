use std::{collections::HashMap, fmt::Debug};

use crate::chunk_mesh_builder::ChunkMesh;
use shared::entities::{Chunk, ChunkPos};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum BuilderError {
    #[error("Chunk already exists at position: {0:?}")]
    ChunkAlreadyExists(ChunkPos),
}

pub trait ChunkBuilder<T: Send + Sync + Default>: Debug + Send + Sync {
    /// Always adds a chunk to the builder.
    /// If chunk with the same position is already inside builder, it will be overwritten.
    /// Additional data can be used to store version or other metadata.
    fn force_build(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, additional_data: T);

    /// Adds a chunk to the builder.
    /// If chunk with the same position is already inside builder, the function will return an error.
    /// Additional data can be used to store version or other metadata.
    fn try_build(
        &mut self,
        chunk_pos: ChunkPos,
        chunk: &Chunk,
        additional_data: T,
    ) -> Result<(), BuilderError>;

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
    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, T)>;
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
