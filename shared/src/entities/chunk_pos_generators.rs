use crate::entities::{ChunkPos, CHUNK_SIZE};

/// Generates square of chunk positions around center.
/// Generated square is 2D (without height (Y axis)).
#[derive(Debug, Clone)]
pub struct ChunkPosGenerator2D {
    current: ChunkPos,
    start: ChunkPos,
    end: ChunkPos,
    remaining: usize,
}

impl ChunkPosGenerator2D {
    /// `center` - center of generated square.
    /// `dist` - distance in chunks (0,1,2) from center to edge of generated square.
    pub fn new(center: ChunkPos, dist: usize) -> Self {
        let dist = dist * CHUNK_SIZE;
        // 0, 16, 32 -> 0, 1, 2
        let (cx, cz) = (center.x, center.z);
        let start_x = cx - dist as isize;
        let start_z = cz - dist as isize;
        let end_x = cx + dist as isize;
        let end_z = cz + dist as isize;

        let chunks_per_axis = dist / CHUNK_SIZE * 2 + 1;
        let remaining = chunks_per_axis.pow(2);

        Self {
            remaining,
            current: ChunkPos::new(start_x, 0, start_z),
            start: ChunkPos::new(start_x, 0, start_z),
            end: ChunkPos::new(end_x, 0, end_z),
        }
    }
}

impl Iterator for ChunkPosGenerator2D {
    type Item = ChunkPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.z > self.end.z {
            return None;
        }

        let result = self.current;

        self.current.x += CHUNK_SIZE as isize;
        if self.current.x > self.end.x {
            self.current.x = self.start.x;
            self.current.z += CHUNK_SIZE as isize;
        }

        self.remaining -= 1;
        Some(result)
    }
}

impl ExactSizeIterator for ChunkPosGenerator2D {
    fn len(&self) -> usize {
        self.remaining
    }
}

/// Generates square of chunk positions around center.
/// Generated square is 3D (with height (Z axis)).
#[derive(Debug, Clone)]
pub struct ChunkPosGenerator3D {
    current: ChunkPos,
    start: ChunkPos,
    end: ChunkPos,
    remaining: usize,
}

impl ChunkPosGenerator3D {
    /// `center` - center of generated square
    /// `dist` - distance in chunks (0,1,2) of generated square
    pub fn new(center: ChunkPos, dist: usize) -> Self {
        let dist = dist * CHUNK_SIZE;
        // 0, 16, 32 -> 0, 1, 2
        let (cx, cy, cz) = (center.x, center.y, center.z);
        let start_x = cx - dist as isize;
        let start_y = cy - dist as isize;
        let start_z = cz - dist as isize;
        let end_x = cx + dist as isize;
        let end_y = cy + dist as isize;
        let end_z = cz + dist as isize;

        let chunks_per_axis = dist / CHUNK_SIZE * 2 + 1;
        let remaining = chunks_per_axis.pow(3);

        Self {
            remaining,
            current: ChunkPos::new(start_x, start_y, start_z),
            start: ChunkPos::new(start_x, start_y, start_z),
            end: ChunkPos::new(end_x, end_y, end_z),
        }
    }
}

impl Iterator for ChunkPosGenerator3D {
    type Item = ChunkPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.z > self.end.z {
            return None;
        }

        let result = self.current;

        self.current.x += CHUNK_SIZE as isize;
        if self.current.x > self.end.x {
            self.current.x = self.start.x;
            self.current.y += CHUNK_SIZE as isize;

            if self.current.y > self.end.y {
                self.current.y = self.start.y;
                self.current.z += CHUNK_SIZE as isize;
            }
        }

        self.remaining -= 1;
        Some(result)
    }
}

impl ExactSizeIterator for ChunkPosGenerator3D {
    fn len(&self) -> usize {
        self.remaining
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_generate_chunk_pos_2d() {
        let generator = ChunkPosGenerator2D::new(ChunkPos::new(0, 0, 0), 2);
        assert_eq!(generator.len(), 25);

        let actual_positions: HashSet<ChunkPos> = generator.collect();
        let expected_positions: HashSet<ChunkPos> = (-2..=2)
            .flat_map(|z| {
                (-2..=2).map(move |x| {
                    ChunkPos::new(x * CHUNK_SIZE as isize, 0, z * CHUNK_SIZE as isize)
                })
            })
            .collect();

        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn test_generate_chunk_pos_3d() {
        let generator = ChunkPosGenerator3D::new(ChunkPos::new(0, 0, 0), 2);
        assert_eq!(generator.len(), 125);

        let actual_positions: HashSet<ChunkPos> = generator.collect();
        let expected_positions: HashSet<_> = (-2..=2)
            .flat_map(|z| {
                (-2..=2).flat_map(move |y| {
                    (-2..=2).map(move |x| {
                        ChunkPos::new(
                            x * CHUNK_SIZE as isize,
                            y * CHUNK_SIZE as isize,
                            z * CHUNK_SIZE as isize,
                        )
                    })
                })
            })
            .collect();

        assert_eq!(actual_positions, expected_positions);
    }
}
