use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use serde_json::Value;

use super::{binary, integers, Generate};

pub struct RandomsGenerator {
    use_true_random: bool,
}

impl RandomsGenerator {
    pub fn use_true_random(mut self) -> Self {
        self.use_true_random = true;
        self
    }
}

impl Generate<HegelRandom> for RandomsGenerator {
    fn schema(&self) -> Option<Value> {
        None // Always compositional - no single schema describes this
    }

    fn generate(&self) -> HegelRandom {
        if self.use_true_random {
            let seed: u64 = integers().generate();
            HegelRandom::True(Box::new(StdRng::seed_from_u64(seed)))
        } else {
            HegelRandom::Artificial
        }
    }
}

pub fn randoms() -> RandomsGenerator {
    RandomsGenerator {
        use_true_random: false,
    }
}

pub enum HegelRandom {
    Artificial,
    True(Box<StdRng>),
}

impl RngCore for HegelRandom {
    fn next_u32(&mut self) -> u32 {
        match self {
            Self::Artificial => integers().generate(),
            Self::True(rng) => rng.next_u32(),
        }
    }

    fn next_u64(&mut self) -> u64 {
        match self {
            Self::Artificial => integers().generate(),
            Self::True(rng) => rng.next_u64(),
        }
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        match self {
            Self::Artificial => {
                let bytes: Vec<u8> = binary()
                    .with_min_size(dest.len())
                    .with_max_size(dest.len())
                    .generate();
                dest.copy_from_slice(&bytes);
            }
            Self::True(rng) => rng.fill_bytes(dest),
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
