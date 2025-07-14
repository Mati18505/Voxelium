use std::cmp::min;

use crate::entities::ChunkPos;

pub struct PendingChunkQueue {
    pending_chunks: Vec<ChunkPos>
}

impl PendingChunkQueue {
    pub fn new() -> Self {
        Self {
            pending_chunks: Vec::new(),
        }
    }
    pub fn add_chunk(&mut self, pos: ChunkPos) {
        self.pending_chunks.push(pos);
    }

    pub fn take_nearest_chunks(&mut self, k: usize, player_pos: ChunkPos) -> Vec<ChunkPos> {
        let k = min(k, self.pending_chunks.len());
        self.move_k_nearest_chunks_to_back(k, player_pos);

        let median = self.pending_chunks.len() - k;
        let nearest_chunks: Vec<ChunkPos> = self.pending_chunks[median..].to_vec();

        self.pending_chunks.truncate(median);

        nearest_chunks
    }

    fn move_k_nearest_chunks_to_back(&mut self, k: usize, player_pos: ChunkPos) {
        if self.pending_chunks.is_empty() {
            return
        }

        let index = self.pending_chunks.len().saturating_sub(k);

        self.pending_chunks.select_nth_unstable_by_key(index, |chunk_pos| {
            let distance = Self::chunk_pos_distance_sq(player_pos, *chunk_pos);

            -distance
        });
    }

    fn chunk_pos_distance_sq(a: ChunkPos, b: ChunkPos) -> isize {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let dz = a.z - b.z;

        dx*dx + dy*dy + dz*dz
    }
}