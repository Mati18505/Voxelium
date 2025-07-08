use crate::entities::{BlockID, BlockPos, BlockTypeStorage, World};
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

pub struct RaycastConfig<'a> {
    pub world: &'a World, 
    pub block_type_storage: &'a BlockTypeStorage, 
    pub range: f32,
    pub increment: f32,
}

pub fn raycast(start: Vector3<f32>, dir: Vector3<f32>, config: &RaycastConfig) -> RaycastResult {
    assert!(is_normalized(dir), "Direction must be normalized.");
    assert!(config.range >= 0.0, "Range must be positive.");
    assert!(config.increment > 0.0, "Increment must be greater than zero.");

    let mut curr_pos = start;
    let mut raycast_result = RaycastResult::default();
    let mut previous_block_id: BlockID = config.world.get_block(f32_pos_to_block_pos(start)).unwrap_or_default();

    while curr_pos.distance2(start) <= config.range * config.range && !raycast_result.collide {
        let curr_block_pos = f32_pos_to_block_pos(curr_pos);

        if let Some(block_id) = config.world.get_block(curr_block_pos).ok() {
            if let Some(block_type) = config.block_type_storage.get_by_id(block_id) {

                if block_type.affect_raycast {
                    raycast_result.collide = true;
                    raycast_result.hitpoint = Hitpoint {
                        pos: curr_block_pos,
                        block_id: block_id,
                    };
                    raycast_result.step_before_hitpoint = Hitpoint {
                        pos: f32_pos_to_block_pos(curr_pos - dir * config.increment),
                        block_id: previous_block_id,
                    }
                }
            }

            previous_block_id = block_id;
        } else {
            previous_block_id = BlockID::default();
        }

        curr_pos += dir * config.increment;
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
    use super::*;

    #[test]
    fn test_raycast_empty_world() {
        let start = Vector3::new(0.0, 0.0, 0.0);
        let dir = Vector3::new(1.0, 0.0, 0.0);

        let config = RaycastConfig {
            world: &World::default(),
            block_type_storage: &BlockTypeStorage::default(),
            range: 20.0,
            increment: 0.01,
        };

        let result = raycast(start, dir, &config);

        assert_eq!(result.collide, false);
    }
}