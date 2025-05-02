use voxelium::entities::{Chunk, ChunkPos};
use super::{Generator, RandomNoise};

pub fn load_chunk(pos: ChunkPos) -> Chunk {
    generate_chunk(pos)
}

fn generate_chunk(pos: ChunkPos) -> Chunk {
    let noise = Box::new(RandomNoise::new());
    let generator = Generator::new(pos, noise).generate_terrain();

    Chunk::new(generator.get())
}