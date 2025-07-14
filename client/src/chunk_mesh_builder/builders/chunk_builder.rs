use std::collections::HashMap;
use shared::entities::{Chunk, ChunkPos};
use super::chunk_mesh_builder::ChunkMesh;

pub trait ChunkBuilder<T: Send + Sync> {
    /// Adds a chunk to the builder.
    /// Additional data can be used to store version or other metadata.
    /// If chunk with the same position exists in builder, or is built, it will be replaced.
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk, additional_data: Option<T>);

    /// Updates the builder state based on the player's position.
    /// Should be called once per frame.
    /// Can optimize chunk building based on player position.
    fn update(&mut self, player_pos: ChunkPos);

    /// Removes a chunk from the builder, cancelling build operation.
    fn remove_chunk(&mut self, chunk_pos: ChunkPos);

    /// Removes all chunks from the builder, cancelling all build operations.
    fn clear_all(&mut self);

    /// Returns a map of all built chunks and additional data.
    /// Additional data is none only if chunk was added without additional data.
    /// Removes chunk from the builder.
    fn poll_completed(&mut self) -> HashMap<ChunkPos, (ChunkMesh, Option<T>)>;
}

pub trait VersionedChunkBuilder {
    /// Adds a chunk to the builder. 
    /// If chunk with the same position exists in builder, or is built, it will be replaced.
    /// Newest call to this function equals the latest version of the chunk.
    fn build_chunk(&mut self, chunk_pos: ChunkPos, chunk: &Chunk);

    /// Updates the builder state based on the player's position.
    /// Should be called once per frame.
    fn update(&mut self, player_pos: ChunkPos);

    /// Returns true if a chunk is built with the latest version.
    fn is_chunk_with_latest_version_built(&self, chunk_pos: ChunkPos) -> bool;

    /// Removes all chunks from the builder, cancelling all build operations.
    fn clear_all(&mut self);

    /// Returns a chunk built with the latest version.
    /// If latest version is different than version of the chunk, or no chunk is built, it will return None.
    /// Restarts chunk versioning for the chunk and removes chunk from the builder.
    fn take_chunk_built_with_latest_version(&mut self, chunk_pos: ChunkPos) -> Option<ChunkMesh>;
}