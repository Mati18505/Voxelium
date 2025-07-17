use bevy::math::Vec3;
use cgmath::Vector3;
use shared::{
    entities::BlockTypeStorage,
    physics::{raycast, RaycastConfig, RaycastResult},
};

pub fn raycast_from_controller(
    controller_pos: Vec3,
    controller_forward: Vec3,
    world: &shared::entities::World,
    server_block_type_storage: &BlockTypeStorage,
) -> RaycastResult {
    // Convert bevy direction to our direction
    let mut start = Vector3::new(controller_pos.x, -controller_pos.z, controller_pos.y);
    let dir = Vector3::new(
        controller_forward.x,
        -controller_forward.z,
        controller_forward.y,
    );

    let config = RaycastConfig {
        world: world,
        block_type_storage: server_block_type_storage,
        range: 16.0,
        increment: 0.01,
    };

    start -= dir * config.increment;

    raycast(start, dir, &config)
}
