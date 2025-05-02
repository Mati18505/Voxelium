use voxelium::entities::{Chunk, ChunkPos};
use super::{generator::Noise, Generator, RandomNoise};

pub struct ChunkLoader {
    noise_factory: Box<dyn NoiseFactory>,
}

impl ChunkLoader {
    pub fn new(noise_factory: Box<dyn NoiseFactory>) -> Self {
        ChunkLoader { 
            noise_factory
        }
    }

    pub fn load_chunk(&mut self, pos: ChunkPos) -> Chunk {
        self.generate_chunk(pos)
    }

    fn generate_chunk(&mut self, pos: ChunkPos) -> Chunk {
        let noise = self.noise_factory.create_noise();
        let generator = Generator::new(pos, noise).generate_terrain();

        Chunk::new(generator.get())
    }
}

impl Default for ChunkLoader {
    fn default() -> Self {
        ChunkLoader::new(Box::new(RandomNoiseFactory))
    }
}

pub trait NoiseFactory {
    fn create_noise(&self) -> Box<dyn Noise<isize>>;
}

pub struct RandomNoiseFactory;
impl NoiseFactory for RandomNoiseFactory {
    fn create_noise(&self) -> Box<dyn Noise<isize>> {
        Box::new(RandomNoise::new())
    }
}


#[cfg(test)]
mod test {
    use voxelium::entities::*;
    use super::*;

    struct TestNoise;
    impl Noise<isize> for TestNoise {
        fn gen_range(&mut self, _range: std::ops::Range<isize>) -> isize {
            16
        }
    }

    struct TestNoiseFactory;
    impl NoiseFactory for TestNoiseFactory {
        fn create_noise(&self) -> Box<dyn Noise<isize>> {
            return Box::new(TestNoise);
        }
    }

    #[test]
    fn test_generate_chunk() {
        let mut chunk_loader = ChunkLoader::new(Box::new(TestNoiseFactory));
        let pos = ChunkPos::new(0, 0, 0);
        let chunk = chunk_loader.generate_chunk(pos);

        let expected_blocks = vec![16; CHUNK_SIZE.pow(3)];
        let expected = Chunk::new(BlockStorage::new(expected_blocks));

        assert_eq!(chunk, expected);
    }
}