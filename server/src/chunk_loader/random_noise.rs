use std::ops::Range;

use super::generator::Noise;
use rand::{rngs::ThreadRng, Rng};

#[derive(Debug, Clone)]
pub struct RandomNoise {
    engine: ThreadRng,
}

impl RandomNoise {
    pub fn new() -> Self {
        RandomNoise {
            engine: rand::rng(),
        }
    }
}

impl Noise<i64> for RandomNoise {
    fn gen_range(&mut self, range: Range<i64>) -> i64 {
        self.engine.random_range(range)
    }
}
