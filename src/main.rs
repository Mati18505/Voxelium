use cgmath::Vector3;
use project_craft_server::world;
use project_craft_server::types::{ChunkPos};
use project_craft_server::chunk_loader;

fn main() {
    let mut world = world::World::new();
    let pos = ChunkPos(Vector3::<isize>::new(0, 0, 0));
    world.add_chunk(ChunkPos(pos.clone()), chunk_loader::load_chunk(&pos));
    let result = world.get_chunk(&pos);
    dbg!(result);
}
