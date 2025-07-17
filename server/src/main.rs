use std::fmt;

use shared::entities::{world, Chunk, ChunkPos, ChunkRepository};

mod chunk_io;
use chunk_io::ChunkLoader;

struct DebugChunk {
    chunk: Chunk,
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
    let mut chunk_loader = ChunkLoader::default();
    let mut world = world::World::new();
    let pos = ChunkPos::new(0, 0, 0);

    world.set_chunk(pos, chunk_loader.load_chunk(pos));

    let result = world.get_chunk(pos);

    match result {
        Some(chunk) => println!("{}", DebugChunk::new(chunk.clone())),
        None => println!("Chunk don't exist."),
    }
}
