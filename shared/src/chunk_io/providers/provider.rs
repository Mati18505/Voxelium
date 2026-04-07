use crate::entities::{BlockRegistry, Chunk, ChunkPos};

pub trait ChunkProvider: Send + Sync {
    fn load_chunk(&mut self, pos: ChunkPos, registry: &dyn BlockRegistry) -> Chunk;
}
