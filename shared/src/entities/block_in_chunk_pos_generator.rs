use crate::entities::{BlockInChunkPos, CHUNK_SIZE};

/// Generates all BlockInChunkPos positions in chunk, 
/// Positions are ordered in a way, that BlockInChunkPos.index() indexes them in optimal (cache-friendly) way.
#[derive(Debug, Clone)]
pub struct BlockInChunkPosGenerator {
    current: BlockInChunkPos,
    remaining: usize,
}

impl BlockInChunkPosGenerator {
    pub fn new() -> Self {
        Self {
            current: BlockInChunkPos::new(0, 0, 0),
            remaining: CHUNK_SIZE.pow(3),
        }
    }
}

impl Iterator for BlockInChunkPosGenerator {
    type Item = BlockInChunkPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
              
        let result = self.current;

        self.current.x += 1;
        if self.current.x == CHUNK_SIZE {
            self.current.x = 0;
            self.current.y += 1;

            if self.current.y == CHUNK_SIZE {
                self.current.y = 0;
                self.current.z += 1;
            }
        }

        self.remaining -= 1;
        Some(result)
    }
}

impl ExactSizeIterator for BlockInChunkPosGenerator {
    fn len(&self) -> usize {
        self.remaining
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_generator_produces_all_unique_positions() {
        let positions: HashSet<BlockInChunkPos> = BlockInChunkPosGenerator::new().collect();
        assert_eq!(positions.len(), CHUNK_SIZE.pow(3));
    }

    /// Tests that the positions returned by the generator match the order of indicies returned by `pos.index()`, ensuring a cache-friendly memory layout.
    #[test]
    fn test_generator_index_order_matches_linear_memory_layout() {
        let mut generator = BlockInChunkPosGenerator::new();
        assert_eq!(generator.len(), CHUNK_SIZE.pow(3));

        for (i, pos) in generator.enumerate() {
            assert_eq!(pos.index(), i);
        }
    }

    #[test]
    fn test_generator_matches_from_index() {
        let generator = BlockInChunkPosGenerator::new();

        for (i, pos) in generator.enumerate() {
            assert_eq!(BlockInChunkPos::from_index(i), pos);
        }
    }
}