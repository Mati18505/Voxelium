use cgmath::Vector3;
use voxelium::chunk::Chunk;
use voxelium::{terrain_generator, world};
use voxelium::types::ChunkPos;
use voxelium::chunk_loader;

fn main() {
    let mut world = world::World::new();
    let pos = ChunkPos(Vector3::<isize>::new(0, 0, 0));
    world.add_chunk(ChunkPos(pos.clone()), chunk_loader::load_chunk(&pos));
    let result = world.get_chunk(&pos);
    dbg!(result);
}

#[test]
fn test_adding_chunks() {
    let mut world = world::World::new();
    let pos = ChunkPos(Vector3::<isize>::new(0, 0, 0));
    let blocks = terrain_generator::generate(&pos);
    let chunk = Chunk::new(blocks);

    world.add_chunk(ChunkPos(pos.clone()), chunk.clone());

    let chunk2 = world.get_chunk_cloned(&pos).expect("cannot get chunk from world");
    assert_eq!(chunk, chunk2);
}
