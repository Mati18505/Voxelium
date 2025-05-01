use cgmath::Vector3;
use voxelium::{chunk_loader, entities::{world, BlockPos, Chunk, ChunkPos}, terrain_generator};

fn main() {
    let mut world = world::World::new();
    let pos = ChunkPos(Vector3::<isize>::new(0, 0, 0));
    world.add_chunk(ChunkPos(*pos), chunk_loader::load_chunk(&pos));
    let result = world.get_chunk(&pos);
    dbg!(result);
}

#[test]
fn test_adding_chunks() {
    let mut world = world::World::new();
    let pos = ChunkPos(Vector3::<isize>::new(0, 0, 0));
    let blocks = terrain_generator::generate(&pos);
    let chunk = Chunk::new(blocks);

    world.add_chunk(ChunkPos(*pos), chunk.clone());

    let chunk2 = world.get_chunk(&pos).expect("cannot get chunk from world");
    assert_eq!(&chunk, chunk2);
}
