use std::{cmp, collections::HashMap, fmt};

use crate::entities::ChunkPos;

pub struct PendingChunkQueue {
    pending_chunks: Vec<ChunkPos>,
    index_map: HashMap<ChunkPos, usize>,
}

impl PendingChunkQueue {
    pub fn new() -> Self {
        Self {
            pending_chunks: Vec::new(),
            index_map: HashMap::new(),
        }
    }

    /// Adds a chunk position to the queue.
    /// If the chunk is already in the queue, it will not be added again.
    pub fn add_chunk(&mut self, pos: ChunkPos) {
        if self.index_map.contains_key(&pos) {
            return;
        }

        self.pending_chunks.push(pos);
        self.index_map.insert(pos, self.pending_chunks.len() - 1);

        self.check_invariants();
    }

    /// Removes a chunk position from the queue.
    /// If the chunk is not in the queue, nothing happens.
    pub fn remove_chunk(&mut self, pos: ChunkPos) {
        if let Some(index) = self.index_map.remove(&pos) {
            self.pending_chunks.swap_remove(index);

            if let Some(last) = self.pending_chunks.get(index) {
                self.index_map.insert(*last, index);
            }
        }

        self.check_invariants();
    }

    /// Removes and returns up to `k` chunks that are nearest to `player_pos`.
    /// The returned chunks are guaranteed to be among the `k` nearest in the queue, but their order is not guaranteed.
    /// If there are fewer than `k` chunks, returns all of them.
    pub fn take_nearest_chunks(&mut self, k: usize, player_pos: ChunkPos) -> Vec<ChunkPos> {
        let k = k.min(self.pending_chunks.len());
        self.move_k_nearest_chunks_to_back(k, player_pos);

        let median = self.pending_chunks.len() - k;
        let nearest_chunks: Vec<ChunkPos> = self.pending_chunks.split_off(median);

        self.rebuild_index_map();

        nearest_chunks
    }

    /// This doesn't rebuild the index map.
    fn move_k_nearest_chunks_to_back(&mut self, k: usize, player_pos: ChunkPos) {
        if self.pending_chunks.is_empty() || k == 0 {
            return;
        }

        let index = self.pending_chunks.len().saturating_sub(k);

        self.pending_chunks
            .select_nth_unstable_by_key(index, |chunk_pos| {
                let distance = Self::chunk_pos_distance_sq(player_pos, *chunk_pos);

                cmp::Reverse(distance)
            });
    }

    fn rebuild_index_map(&mut self) {
        self.index_map.clear();

        for (i, pos) in self.pending_chunks.iter().enumerate() {
            self.index_map.insert(*pos, i);
        }

        self.check_invariants();
    }

    fn chunk_pos_distance_sq(a: ChunkPos, b: ChunkPos) -> usize {
        let dx = (a.x - b.x) as i64;
        let dy = (a.y - b.y) as i64;
        let dz = (a.z - b.z) as i64;

        (dx * dx + dy * dy + dz * dz) as usize
    }

    fn check_invariants(&self) {
        debug_assert_eq!(self.pending_chunks.len(), self.index_map.len());

        for (i, pos) in self.pending_chunks.iter().enumerate() {
            debug_assert_eq!(self.index_map.get(pos), Some(&i));
        }
    }
}

impl Default for PendingChunkQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PendingChunkQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingChunkQueue")
            .field("pending_chunks", &self.pending_chunks.len())
            .field("index_map", &self.index_map.len())
            .finish()
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

    fn validate(chunks: &[ChunkPos], expected: &HashSet<ChunkPos>) {
        assert_eq!(chunks.len(), expected.len());
        assert!(chunks.iter().all(|pos| expected.contains(pos)));
        assert!(expected.iter().all(|pos| chunks.contains(pos)));
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

        validate(&nearest_chunks, &expected_nearest);

        let expected_remaining =
            HashSet::from([ChunkPos::new(48, 48, 48), ChunkPos::new(64, 64, 64)]);

        validate(&queue.pending_chunks, &expected_remaining);
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

        let expected_front = HashSet::from([ChunkPos::new(48, 48, 48), ChunkPos::new(64, 64, 64)]);
        let expected_back = HashSet::from([
            ChunkPos::new(16, 16, 16),
            ChunkPos::new(32, 32, 32),
            ChunkPos::new(0, 0, 0),
        ]);

        validate(&queue.pending_chunks[..median], &expected_front);
        validate(&queue.pending_chunks[median..], &expected_back);
    }

    #[test]
    fn test_chunk_pos_distance_sq() {
        let a = ChunkPos::new(0, 0, 0);
        let b = ChunkPos::new(48, 64, 0);
        assert_eq!(
            PendingChunkQueue::chunk_pos_distance_sq(a, b),
            48 * 48 + 64 * 64
        );

        let c = ChunkPos::new(-48, -64, 0);
        assert_eq!(
            PendingChunkQueue::chunk_pos_distance_sq(a, c),
            48 * 48 + 64 * 64
        );

        let d = ChunkPos::new(16, 16, 16);
        assert_eq!(
            PendingChunkQueue::chunk_pos_distance_sq(a, d),
            16 * 16 + 16 * 16 + 16 * 16
        );
    }

    #[test]
    fn test_add_chunk() {
        let mut queue = PendingChunkQueue::new();

        queue.add_chunk(ChunkPos::new(16, 16, 16));
        queue.add_chunk(ChunkPos::new(16, 16, 16));

        assert_eq!(queue.pending_chunks.len(), 1);
        assert!(queue.index_map.contains_key(&ChunkPos::new(16, 16, 16)));
    }

    #[test]
    fn test_remove_chunk() {
        let mut queue = create_queue();
        queue.remove_chunk(ChunkPos::new(16, 16, 16));

        assert!(!queue.index_map.contains_key(&ChunkPos::new(16, 16, 16)));
    }

    #[test]
    fn test_remove_non_existent_chunk() {
        let mut queue = create_queue();
        queue.remove_chunk(ChunkPos::new(128, 128, 128));

        assert!(!queue.index_map.contains_key(&ChunkPos::new(128, 128, 128)));
    }

    #[test]
    fn test_take_and_remove() {
        let mut queue = create_queue();
        assert_eq!(queue.pending_chunks.len(), 5);

        let player_pos = ChunkPos::new(0, 0, 0);

        let _ = queue.take_nearest_chunks(3, player_pos);
        queue.remove_chunk(ChunkPos::new(48, 48, 48));

        let expected = HashSet::from([ChunkPos::new(64, 64, 64)]);

        validate(&queue.pending_chunks, &expected);
    }

    #[test]
    fn test_remove_last_chunk() {
        let mut queue = PendingChunkQueue::new();
        queue.add_chunk(ChunkPos::new(64, 64, 64));

        queue.remove_chunk(ChunkPos::new(64, 64, 64));

        assert_eq!(queue.pending_chunks.len(), 0);
        assert_eq!(queue.index_map.len(), 0);
    }
}
