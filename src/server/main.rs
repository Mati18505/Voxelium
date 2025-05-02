use std::fmt;

use cgmath::Vector3;
use voxelium::{chunk_loader, entities::{world, BlockPos, Chunk, ChunkPos}, terrain_generator};

struct DebugChunk {
    chunk: Chunk
}

impl DebugChunk {
    fn new(chunk: Chunk) -> Self {
        DebugChunk { chunk }
    }
}

impl fmt::Display for DebugChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const COLS: i32 = 32;
        let mut out: String = "".to_string();
        let mut counter = 0;

        for i in self.chunk.get_block_storage().iter() {
            if counter == 0 {
                out += "\r\n";
            }

            out += &i.to_string();

            counter += 1;
            counter = counter % COLS;
        } 

        write!(f, "{}", out).unwrap();
        Ok(())
    }
}

fn main() {
    let mut world = world::World::new();
    let pos = ChunkPos(Vector3::<isize>::new(0, 0, 0));
    world.add_chunk(ChunkPos(*pos), chunk_loader::load_chunk(&pos));
    let result = world.get_chunk(&pos);
    
    match result {
        Some(chunk) => println!("{}", DebugChunk::new(chunk.clone())),
        None => println!("Chunk don't exist."),
    }
    
}

#[test]
fn test_adding_chunks() {
    let mut world = world::World::new();
    let pos = ChunkPos(Vector3::<isize>::new(0, 0, 0));
    let blocks = terrain_generator::generate(&pos);
    let chunk = Chunk::new(blocks);

    world.add_chunk(ChunkPos(*pos), chunk.clone());
    let world_pos = BlockPos(Vector3::new(0,0,15));
    world.get_block(&world_pos).unwrap();

    let chunk2 = world.get_chunk(&pos).expect("cannot get chunk from world");
    assert_eq!(&chunk, chunk2);
}
