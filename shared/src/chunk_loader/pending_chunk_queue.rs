use std::cmp::{self, min};

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
        let nearest_chunks: Vec<ChunkPos> = self.pending_chunks.split_off(median);

        self.pending_chunks.truncate(median);

        nearest_chunks
    }

    fn move_k_nearest_chunks_to_back(&mut self, k: usize, player_pos: ChunkPos) {
        if self.pending_chunks.is_empty() || k == 0 {
            return
        }

        let index = self.pending_chunks.len().saturating_sub(k);

        self.pending_chunks.select_nth_unstable_by_key(index, |chunk_pos| {
            let distance = Self::chunk_pos_distance_sq(player_pos, *chunk_pos);

            cmp::Reverse(distance)
        });
    }

    fn chunk_pos_distance_sq(a: ChunkPos, b: ChunkPos) -> isize {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        let dz = a.z - b.z;

        dx*dx + dy*dy + dz*dz
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn create_queue() -> PendingChunkQueue {
        let mut queue = PendingChunkQueue::new();

        queue.add_chunk(ChunkPos::new(0, 0, 0));
        queue.add_chunk(ChunkPos::new(16, 16, 16));
        queue.add_chunk(ChunkPos::new(32, 32, 32));
        queue.add_chunk(ChunkPos::new(48, 48, 48));
        queue.add_chunk(ChunkPos::new(64, 64, 64));

        queue
    }
    
    #[test]
    fn test_pending_chunk_queue() {
        let mut queue = create_queue();
        assert_eq!(queue.pending_chunks.len(), 5);

        let player_pos = ChunkPos::new(16, 16, 16);

        let nearest_chunks = queue.take_nearest_chunks(3, player_pos);
        let expected_nearest = HashSet::from([
            ChunkPos::new(16, 16, 16),
            ChunkPos::new(32, 32, 32),
            ChunkPos::new(0, 0, 0),
        ]);

        assert_eq!(nearest_chunks.len(), 3);
        assert!(nearest_chunks.iter().all(|pos| expected_nearest.contains(pos)));

        let expected_remaining = HashSet::from([
            ChunkPos::new(48, 48, 48),
            ChunkPos::new(64, 64, 64),
        ]);
        assert_eq!(queue.pending_chunks.len(), 2);
        assert!(queue.pending_chunks.iter().all(|pos| expected_remaining.contains(pos)));
    }

    #[test]
    fn test_pending_chunk_queue_empty() {
        let mut queue = PendingChunkQueue::new();
        assert!(queue.pending_chunks.is_empty());

        let player_pos = ChunkPos::new(0, 0, 0);
        let nearest_chunks = queue.take_nearest_chunks(3, player_pos);
        assert!(nearest_chunks.is_empty());
    }

    #[test]
    fn test_pending_chunk_queue_single() {
        let mut queue = PendingChunkQueue::new();
        queue.add_chunk(ChunkPos::new(16, 16, 16));
        assert_eq!(queue.pending_chunks.len(), 1);

        let player_pos = ChunkPos::new(0, 0, 0);
        let nearest_chunks = queue.take_nearest_chunks(1, player_pos);
        assert_eq!(nearest_chunks.len(), 1);
        assert_eq!(nearest_chunks[0], ChunkPos::new(16, 16, 16));
    }
    
    #[test]
    fn test_pending_chunk_queue_k_zero() {
        let mut queue = create_queue();
        assert_eq!(queue.pending_chunks.len(), 5);

        let player_pos = ChunkPos::new(16, 16, 16);
        let nearest_chunks = queue.take_nearest_chunks(0, player_pos);
        assert!(nearest_chunks.is_empty());
        assert_eq!(queue.pending_chunks.len(), 5);
    }

    #[test]
    fn test_move_k_nearest_chunks_to_back() {
        let mut queue = create_queue();
        assert_eq!(queue.pending_chunks.len(), 5);
        let median = queue.pending_chunks.len() - 3;

        let player_pos = ChunkPos::new(16, 16, 16);
        queue.move_k_nearest_chunks_to_back(3, player_pos);

        let expected_front = HashSet::from([
            ChunkPos::new(48, 48, 48),
            ChunkPos::new(64, 64, 64),
        ]);
        let expected_back = HashSet::from([
            ChunkPos::new(16, 16, 16),
            ChunkPos::new(32, 32, 32),
            ChunkPos::new(0, 0, 0),
        ]);

        assert!(queue.pending_chunks[..median].iter().all(|pos| expected_front.contains(pos)));
        assert!(queue.pending_chunks[median..].iter().all(|pos| expected_back.contains(pos)));
    }

    #[test]
    fn test_chunk_pos_distance_sq() {
        let a = ChunkPos::new(0, 0, 0);
        let b = ChunkPos::new(48, 64, 0);
        assert_eq!(PendingChunkQueue::chunk_pos_distance_sq(a, b), 48*48 + 64*64 + 0*0);

        let c = ChunkPos::new(-48, -64, 0);
        assert_eq!(PendingChunkQueue::chunk_pos_distance_sq(a, c), 48*48 + 64*64 + 0*0);

        let d = ChunkPos::new(16, 16, 16);
        assert_eq!(PendingChunkQueue::chunk_pos_distance_sq(a, d), 16*16 + 16*16 + 16*16);
    }
}