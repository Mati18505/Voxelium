use crate::entities::{Chunk, ChunkPos};

pub trait ChunkProvider: Send + Sync {
    fn load_chunk(&mut self, pos: ChunkPos) -> Chunk;
}
