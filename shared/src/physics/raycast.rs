use crate::entities::{BlockID, BlockPos, BlockTypeStorage, World};
use cgmath::{MetricSpace, Vector3};

pub struct Hitpoint {
    pos: BlockPos,
    block_id: BlockID,
}

impl Default for Hitpoint {
    fn default() -> Self {
        Hitpoint {
            pos: BlockPos { x: 0, y: 0, z: 0 },
            block_id: BlockID::default(),
        }
    }
}

#[derive(Default)]
pub struct RaycastResult {
    collide: bool,
    hitpoint: Hitpoint,
    prev_hitpoint: Hitpoint,
}

pub struct RaycastConfig<'a> {
    world: &'a World, 
    block_type_storage: &'a BlockTypeStorage, 
    range: f32,
}

pub fn raycast(start: Vector3<f32>, dir: Vector3<f32>, config: &RaycastConfig) -> RaycastResult {
    let mut curr_pos = start;
    let mut raycast_result = RaycastResult::default();

    while curr_pos.distance2(start) <= config.range * config.range && !raycast_result.collide {
        let curr_block_pos = f32_pos_to_block_pos(curr_pos);

        if let Some(block_id) = config.world.get_block(curr_block_pos).ok() {
            if let Some(block_type) = config.block_type_storage.get_by_id(block_id) {

                if block_type.affect_raycast {
                    raycast_result.collide = true;
                    raycast_result.prev_hitpoint = raycast_result.hitpoint;
                    raycast_result.hitpoint = Hitpoint {
                        pos: curr_block_pos,
                        block_id: block_id,
                    }
                }
            }
        }

        curr_pos += dir;
    }

    raycast_result
}

fn f32_pos_to_block_pos(pos: Vector3<f32>) -> BlockPos {
    BlockPos {
        x: pos.x.floor() as isize,
        y: pos.y.floor() as isize,
        z: pos.z.floor() as isize,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_raycast_empty_world() {
        let start = Vector3::new(0.0, 0.0, 0.0);
        let dir = Vector3::new(1.0, 0.0, 0.0);

        let config = RaycastConfig {
            world: &World::default(),
            block_type_storage: &BlockTypeStorage::default(),
            range: 20.0,
        };

        let result = raycast(start, dir, &config);

        assert_eq!(result.collide, false);
    }
}