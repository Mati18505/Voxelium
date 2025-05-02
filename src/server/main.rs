use std::fmt;

use voxelium::entities::{world, Chunk, ChunkPos};

mod chunk_loader;

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
            counter %= COLS;
        } 

        write!(f, "{}", out).unwrap();
        Ok(())
    }
}

fn main() {
    let mut world = world::World::new();
    let pos = ChunkPos::new(0, 0, 0);
    world.add_chunk(pos, chunk_loader::load_chunk(pos));
    let result = world.get_chunk(pos);
    
    match result {
        Some(chunk) => println!("{}", DebugChunk::new(chunk.clone())),
        None => println!("Chunk don't exist."),
    }
    
}

#[cfg(test)]
mod test {
    use voxelium::entities::BlockPos;
    use super::chunk_loader::Generator;

    use super::*;

    #[test]
    fn test_adding_chunks() {
        let mut world = world::World::new();
        let pos = ChunkPos::new(0, 0, 0);
        let blocks = Generator::new(pos).generate_terrain().get();
        let chunk = Chunk::new(blocks);

        world.add_chunk(pos, chunk.clone());
        let world_pos = BlockPos::new(0, 0, 15);
        world.get_block(world_pos).unwrap();

        let chunk2 = world.get_chunk(pos).expect("cannot get chunk from world");
        assert_eq!(&chunk, chunk2);
    }
}