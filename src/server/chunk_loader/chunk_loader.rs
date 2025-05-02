use voxelium::entities::{Chunk, ChunkPos};
use super::Generator;

pub fn load_chunk(pos: ChunkPos) -> Chunk {
    generate_chunk(pos)
}

fn generate_chunk(pos: ChunkPos) -> Chunk {
    let generator = Generator::new(pos).generate_terrain();

    Chunk::new(generator.get())
}