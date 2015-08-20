extern crate rand;
use rand::*;

pub struct Soliton {
    n: u32,
    rng: StdRng
}

impl Soliton {
    pub fn new(n: u32, seed: usize) -> Soliton {
        let seedarr: &[_] = &[seed];
        let rng: StdRng = SeedableRng::from_seed(seedarr);
        Soliton {n: n, rng: rng}
    }
}

impl Iterator for Soliton {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        let x = self.rng.gen::<f32>();
        let i = (1.0/x).ceil() as u32;
        if i <= self.n {
            Some(i)
        }
        else {
            Some(1)
        }
    }
}
