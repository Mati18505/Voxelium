use crate::entities::{get_block, BlockID, BlockPos, BlockTypeStorage, ChunkRepository};
use cgmath::{InnerSpace, MetricSpace, Vector3};

#[derive(Debug)]
pub struct Hitpoint {
    pub pos: BlockPos,
    pub block_id: BlockID,
}

impl Default for Hitpoint {
    fn default() -> Self {
        Hitpoint {
            pos: BlockPos { x: 0, y: 0, z: 0 },
            block_id: BlockID::default(),
        }
    }
}

#[derive(Default, Debug)]
pub struct RaycastResult {
    pub collide: bool,
    pub hitpoint: Hitpoint,
    pub step_before_hitpoint: Hitpoint,
}

pub struct RaycastConfig<'a, Chunks: ChunkRepository> {
    pub chunks: &'a Chunks,
    pub block_type_storage: &'a BlockTypeStorage,
    pub range: f32,
    pub increment: f32,
}

pub fn raycast<Chunks: ChunkRepository>(
    start: Vector3<f32>,
    dir: Vector3<f32>,
    config: &RaycastConfig<Chunks>,
) -> RaycastResult {
    assert!(is_normalized(dir), "Direction must be normalized.");
    assert!(config.range >= 0.0, "Range must be positive.");
    assert!(
        config.increment > 0.0,
        "Increment must be greater than zero."
    );

    let mut curr_pos = start;
    let mut raycast_result = RaycastResult::default();
    let mut previous_block_id: BlockID = get_block(f32_pos_to_block_pos(start), config.chunks);
    let mut curr_dir_axis = 0;

    while curr_pos.distance2(start) <= config.range * config.range && !raycast_result.collide {
        let curr_block_pos = f32_pos_to_block_pos(curr_pos);

        let block_id = get_block(curr_block_pos, config.chunks);
        if let Some(block_type) = config.block_type_storage.get_by_id(block_id) {
            if block_type.affect_raycast {
                raycast_result.collide = true;
                raycast_result.hitpoint = Hitpoint {
                    pos: curr_block_pos,
                    block_id,
                };
                raycast_result.step_before_hitpoint = Hitpoint {
                    pos: f32_pos_to_block_pos(curr_pos - dir * config.increment),
                    block_id: previous_block_id,
                }
            }
        }

        previous_block_id = block_id;

        match curr_dir_axis {
            0 => curr_pos.x += dir.x * config.increment,
            1 => curr_pos.y += dir.y * config.increment,
            2 => curr_pos.z += dir.z * config.increment,
            _ => unreachable!(),
        }

        curr_dir_axis += 1;
        curr_dir_axis %= 3;
    }

    raycast_result
}

fn f32_pos_to_block_pos(pos: Vector3<f32>) -> BlockPos {
    BlockPos {
        x: pos.x.round() as isize,
        y: pos.y.round() as isize,
        z: pos.z.round() as isize,
    }
}

fn is_normalized(v: Vector3<f32>) -> bool {
    let length = v.magnitude();
    let epsilon = 1e-6;
    (length - 1.0).abs() < epsilon
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::entities::{Chunk, ChunkPos};

    use super::*;

    #[derive(Default)]
    struct DummyChunkStorage(HashMap<ChunkPos, Chunk>);
    impl ChunkRepository for DummyChunkStorage {
        fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
            self.0.get(&pos)
        }

        fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
            self.0.get_mut(&pos)
        }
    }

    #[test]
    fn test_raycast_empty_world() {
        let chunk_storage = DummyChunkStorage::default();
        let start = Vector3::new(0.0, 0.0, 0.0);
        let dir = Vector3::new(1.0, 0.0, 0.0);

        let config = RaycastConfig {
            chunks: &chunk_storage,
            block_type_storage: &BlockTypeStorage::default(),
            range: 20.0,
            increment: 0.01,
        };

        let result = raycast(start, dir, &config);

        assert!(
            !result.collide,
            "Raycast should not collide in an empty world."
        );
    }
}
