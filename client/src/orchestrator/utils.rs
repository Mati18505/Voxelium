use bevy::math::Vec3;
use cgmath::Vector3;
use shared::{
    entities::{BlockTypeStorage, ChunkRepository},
    physics::{raycast, RaycastConfig, RaycastResult},
};

pub fn raycast_from_controller<Chunks: ChunkRepository>(
    controller_pos: Vec3,
    controller_forward: Vec3,
    chunks: &Chunks,
    server_block_type_storage: &BlockTypeStorage,
) -> RaycastResult {
    let mut start = Vector3::new(controller_pos.x, controller_pos.y, controller_pos.z);
    let dir = Vector3::new(
        controller_forward.x,
        controller_forward.y,
        controller_forward.z,
    );

    let config = RaycastConfig {
        chunks,
        block_type_storage: server_block_type_storage,
        range: 16.0,
        increment: 0.01,
    };

    start -= dir * config.increment;

    raycast(start, dir, &config)
}
