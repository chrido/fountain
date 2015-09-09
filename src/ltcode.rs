use std::vec::Vec;
use std::cell::RefCell;
use std::rc::Rc;
use std::cmp;
use rand::{Rng, sample, StdRng, SeedableRng};

use soliton::RobustSoliton;

pub struct Encoder {
    data: Vec<u8>,
    len: usize,
    blocksize: usize,
    rng: StdRng,
    cnt_blocks: usize,
    sol: RobustSoliton
}

#[derive(Debug)]
pub struct Droplet {
    degree: usize,
    seed: usize,
    data: Vec<u8>
}

impl Droplet {
    fn new(degree: usize, seed: usize, data: Vec<u8>) -> Droplet {
        Droplet {degree: degree, seed: seed, data: data}
    }
}

impl Encoder {
    pub fn new(data: Vec<u8>, blocksize: usize) -> Encoder {
        let mut rng = StdRng::new().unwrap(); //TODO: there should not be any work in the constructor

        let len = data.len();
        let cnt_blocks = ((len as f32)/blocksize as f32).ceil() as usize;
        let sol = RobustSoliton::new(cnt_blocks, rng.gen::<usize>());
        Encoder{data: data, len: len, blocksize: blocksize, rng: rng, cnt_blocks: cnt_blocks, sol: sol}
    }
}

fn get_sample_from_rng_by_seed(seed: usize, n: usize, degree: usize) -> Vec<usize> {
    let seedarr: &[_] = &[seed];
    let mut rng:StdRng = SeedableRng::from_seed(seedarr);
    sample(&mut rng, 0..n, degree)
}

impl Iterator for Encoder {
    type Item = Droplet;

    fn next(&mut self) -> Option<Droplet> {
        let degree = self.sol.next().unwrap() as usize; //TODO: try! macro
        let seed = self.rng.gen::<u32>() as usize;
        let sample = get_sample_from_rng_by_seed(seed, self.cnt_blocks, degree);

        let mut r:Vec<u8> = vec![0; self.blocksize];

        for k in sample {
            let begin = k*self.blocksize;
            let end = cmp::min((k+1)* self.blocksize, self.len);
            let mut j = 0;

            for i in begin..end {
                r[j] ^= self.data[i];
                j +=1;
            }
        }

        Some(Droplet::new(degree, seed, r))
    }
}


pub struct Decoder {
    total_length: usize,
    blocksize: usize,
    unknown_chunks: usize,
    number_of_chunks: usize,
    blocks: Vec<Block>,
    data: Vec<u8>
}

#[derive(Debug)]
pub enum CatchResult {
    Finished(Vec<u8>),
    Missing(usize)
}

#[derive(Debug)]
struct RxDroplet {
    edges_idx: Vec<usize>,
    data: Vec<u8>
}

struct Block {
    idx: usize,
    edges: Vec<Rc<RefCell<RxDroplet>>>,
    begin_at: usize,
    is_known: bool
}


impl Decoder {
    pub fn new(len: usize, blocksize: usize) -> Decoder {
        let data:Vec<u8> = vec![0; len];
        let number_of_chunks = ((len as f32)/blocksize as f32).ceil() as usize;
        let mut edges:Vec<Block> = Vec::with_capacity(number_of_chunks);
        for i in 0..number_of_chunks {
            trace!("d_begin_at: {:?}", i*blocksize);
            let blk = Block{idx: i, edges: Vec::new(), begin_at: blocksize * i, is_known: false};
            edges.push(blk);
        }

        Decoder{ total_length: len, number_of_chunks: number_of_chunks, unknown_chunks: number_of_chunks, blocks: edges, data: data, blocksize: blocksize}
    }

    fn process_droplet(&mut self, droplet: RxDroplet) {
        let mut drops:Vec<Rc<RefCell<RxDroplet>>> = Vec::new();
        drops.push(Rc::new(RefCell::new(droplet)));
        loop {
            match drops.pop() {
                None => return,
                Some(drop) => {
                    let edges = drop.borrow().edges_idx.clone();
                    for ed in edges { //the list is edited, hence we copy first
                        let block = self.blocks.get_mut(ed).unwrap();
                        if block.is_known {
                            let mut b_drop = drop.borrow_mut();
                            for i in 0..self.blocksize {
                                self.data[block.begin_at+i] ^= b_drop.data[i];
                            }
                            let pos = b_drop.edges_idx.iter().position(|x| x == &ed).unwrap();
                            b_drop.edges_idx.remove(pos);
                        }
                        else {
                            block.edges.push(drop.clone());
                        }
                    }
                    if drop.borrow().edges_idx.len() == 1 {
                        let first_idx = drop.borrow().edges_idx.clone().get(0).unwrap().clone();

                        let block = self.blocks.get_mut(first_idx).unwrap();

                        if block.is_known == false {
                            for i in 0..self.blocksize {
                                self.data[block.begin_at+i] = drop.borrow().data[i];
                            }
                            block.is_known = true;
                            self.unknown_chunks -= 1;

                            while block.edges.len() > 0 {
                                let mut to_push = false;
                                let edge = block.edges.pop().unwrap();
                                {
                                    let mut m_edge = edge.borrow_mut();

                                    if m_edge.edges_idx.len() == 1 {
                                        to_push = true;
                                        drops.push(edge.clone());
                                    }
                                    else {
                                        for i in 0..self.blocksize {
                                            m_edge.data[i] ^= self.data[block.begin_at+i]
                                        }

                                        let pos = m_edge.edges_idx.iter().position(|x| x == &block.idx).unwrap();
                                        m_edge.edges_idx.remove(pos);

                                        if m_edge.edges_idx.len() == 1 {
                                            drops.push(edge.clone());
                                       }
                                    }
                                }
                            }
                        }

                    }
                }
            }
        }
    }

    pub fn catch(&mut self, drop: Droplet) -> CatchResult {
        let sample = get_sample_from_rng_by_seed(drop.seed, self.number_of_chunks, drop.degree);
        let rxdrop = RxDroplet {edges_idx: sample, data: drop.data};
        self.process_droplet(rxdrop);
        if self.unknown_chunks == 0 {
            CatchResult::Finished(self.data.clone()) //TODO: there shouldn't be a copy
        }
        else {
            CatchResult::Missing(self.unknown_chunks)
        }
    }
}
