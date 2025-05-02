use std::ops::Range;

use rand::{rngs::ThreadRng, Rng};
use super::generator::Noise;

#[derive(Debug, Clone)]
pub struct RandomNoise {
    engine: ThreadRng
}

impl RandomNoise {
    pub fn new() -> Self {
        RandomNoise { engine: rand::thread_rng() }
    }
}

impl Noise<isize> for RandomNoise {
    fn gen_range(&mut self, range: Range<isize>) -> isize
    {
        self.engine.gen_range(range)
    }
}