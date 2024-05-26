use cgmath::Vector3;

pub fn index(pos: Vector3<usize>, size: usize) -> usize {
    pos.y * size * size + pos.z * size + pos.x
}