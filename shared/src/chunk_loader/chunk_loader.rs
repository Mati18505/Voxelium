use super::{generator::Noise, Generator, RandomNoise};
use crate::entities::{Chunk, ChunkPos};

pub struct ChunkLoader {
    noise_factory: Box<dyn NoiseFactory>,
}

impl ChunkLoader {
    pub fn new(noise_factory: Box<dyn NoiseFactory>) -> Self {
        ChunkLoader { noise_factory }
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

pub trait NoiseFactory: Send + Sync {
    fn create_noise(&self) -> Box<dyn Noise<i64>>;
}

pub struct RandomNoiseFactory;
impl NoiseFactory for RandomNoiseFactory {
    fn create_noise(&self) -> Box<dyn Noise<i64>> {
        Box::new(RandomNoise::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::entities::*;

    struct TestNoise;
    impl Noise<i64> for TestNoise {
        fn gen_range(&mut self, _range: std::ops::Range<i64>) -> i64 {
            16
        }
    }

    struct TestNoiseFactory;
    impl NoiseFactory for TestNoiseFactory {
        fn create_noise(&self) -> Box<dyn Noise<i64>> {
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
