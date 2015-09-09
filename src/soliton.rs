extern crate rand;
use rand::*;

pub struct IdealSoliton {
    n: usize,
    rng: StdRng
}

impl IdealSoliton {
    pub fn new(n: usize, seed: usize) -> IdealSoliton {
        let seedarr: &[_] = &[seed];
        let rng: StdRng = SeedableRng::from_seed(seedarr);
        IdealSoliton {n: n, rng: rng}
    }
}

impl Iterator for IdealSoliton {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let x = self.rng.gen::<f32>();
        let i = (1.0/x).ceil() as usize;
        if i <= self.n {
            Some(i)
        }
        else {
            Some(1)
        }
    }
}
