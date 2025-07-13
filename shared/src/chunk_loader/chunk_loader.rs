use std::{collections::{HashMap, VecDeque}};
use bevy::{prelude::*, tasks::{futures_lite::future, AsyncComputeTaskPool, Task}};

use super::{generator::Noise, Generator, RandomNoise};
use crate::entities::{Chunk, ChunkPos};

pub struct ChunkLoader {
    noise_factory: Box<dyn NoiseFactory>,
    chunks_to_load: VecDeque<ChunkPos>,
    tasks: HashMap<ChunkPos, Task<Chunk>>,
    completed: HashMap<ChunkPos, Chunk>,
}

impl ChunkLoader {
    pub fn new(noise_factory: Box<dyn NoiseFactory>) -> Self {
        ChunkLoader {
            noise_factory,
            chunks_to_load: VecDeque::new(),
            tasks: HashMap::new(),
            completed: HashMap::new(),
        }
    }

    pub fn load_chunk(&mut self, pos: ChunkPos) {
        self.chunks_to_load.push_back(pos);
    }

    pub fn poll_loaded_chunks(&mut self) -> HashMap<ChunkPos, Chunk> {
        std::mem::take(&mut self.completed)
    }

    const MAX_CHUNKS_PER_UPDATE: usize = 16;

    /// Should be called once per frame.
    pub fn update(&mut self) {
        for _ in 0..Self::MAX_CHUNKS_PER_UPDATE {
            if let Some(pos) = self.chunks_to_load.pop_front() {
                let task = self.generate_chunk(pos);

                self.completed.insert(pos, task);
            } else {
                break
            }
        }

        // let completed_tasks = self.poll_completed_tasks();
        // self.completed.extend(completed_tasks);
    }

    fn poll_completed_tasks(&mut self) -> HashMap<ChunkPos, Chunk> {
        let mut completed: HashMap<ChunkPos, Chunk> = HashMap::default();

        for (chunk_pos, mut task) in self.tasks.iter_mut() {
            if let Some(chunk) = future::block_on(future::poll_once(&mut task)) {
                completed.insert(*chunk_pos, chunk);
            }
        }

        for chunk_pos in completed.keys() {
            self.tasks.remove(chunk_pos);
        }

        completed
    }

    fn generate_chunk(&self, pos: ChunkPos) -> Chunk {
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
}
