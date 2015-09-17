use std::vec::Vec;
use std::cell::RefCell;
use std::rc::Rc;
use std::cmp;
use rand::{Rng, sample, StdRng, SeedableRng};

use soliton::IdealSoliton;

/// Encoder for Luby transform codes
pub struct Encoder {
    data: Vec<u8>,
    len: usize,
    blocksize: usize,
    rng: StdRng,
    cnt_blocks: usize,
    sol: IdealSoliton
}


/// A Droplet is created by the Encoder.
#[derive(Debug)]
pub struct Droplet {
    /// How many chunks are used
    degree: usize,
    /// The seed of the Random number generator to create the indexes of the corresponding chunks
    seed: usize,
    /// The payload of the Droplet
    data: Vec<u8>
}

impl Droplet {
    fn new(degree: usize, seed: usize, data: Vec<u8>) -> Droplet {
        Droplet {degree: degree, seed: seed, data: data}
    }
}

impl Encoder {
    /// Constructs a new encoder for Luby transform codes.
    /// In case you send the packages over UDP, the blocksize
    /// should be the MTU size.
    ///
    /// The Encoder implements the iterator. You can use the iterator
    /// to produce an infinte stream of Droplets
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate rand;
    /// extern crate fountaincode;
    ///
    /// fn main() {
    ///     use fountaincode::ltcode::Encoder;
    ///     use self::rand::{thread_rng, Rng};
    ///
    ///     let s:String = thread_rng().gen_ascii_chars().take(1_024).collect();
    ///     let buf = s.into_bytes();
    ///
    ///     let mut enc = Encoder::new(buf, 64);
    ///
    ///     for i in 1..10 {
    ///         println!("droplet {:?}: {:?}", i, enc.next());
    ///     }
    /// }
    /// ```
    pub fn new(data: Vec<u8>, blocksize: usize) -> Encoder {
        let mut rng = StdRng::new().unwrap();

        let len = data.len();
        let cnt_blocks = ((len as f32)/blocksize as f32).ceil() as usize;
        let sol = IdealSoliton::new(cnt_blocks, rng.gen::<usize>());
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

//impl Iterator for SystematicEncoder {
//    type Item = Droplet;
//
//    fn next(&mut self) -> Option<Droplet> {
//
//    }
//}


/// Decoder for the Luby transform
pub struct Decoder {
    total_length: usize,
    blocksize: usize,
    unknown_chunks: usize,
    number_of_chunks: usize,
    cnt_received_drops: usize,
    blocks: Vec<Block>,
    data: Vec<u8>
}

#[derive(Debug)]
pub struct Statistics {
    pub cnt_droplets: usize,
    pub cnt_chunks: usize,
    pub overhead: f32,
    pub unknown_chunks: usize
}

#[derive(Debug)]
pub enum CatchResult {
    Finished(Vec<u8>, Statistics),
    Missing(Statistics)
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
            let blk = Block{idx: i, edges: Vec::new(), begin_at: blocksize * i, is_known: false};
            edges.push(blk);
        }

        Decoder{ total_length: len,
                 number_of_chunks: number_of_chunks,
                 unknown_chunks: number_of_chunks,
                 cnt_received_drops: 0,
                 blocks: edges,
                 data: data,
                 blocksize: blocksize }
    }

    fn process_droplet(&mut self, droplet: RxDroplet) {
        let mut drops:Vec<Rc<RefCell<RxDroplet>>> = Vec::new();
        drops.push(Rc::new(RefCell::new(droplet)));
        loop { //a loop is used instead of recursion
            match drops.pop() {
                None => return,
                Some(drop) => {
                    let edges = drop.borrow().edges_idx.clone();
                    for ed in edges { //the list is edited, hence we copy first
                        let block = self.blocks.get_mut(ed).unwrap();
                        if block.is_known {
                            let mut b_drop = drop.borrow_mut();
                            for i in 0..self.blocksize {
                                b_drop.data[i] ^= self.data[block.begin_at+i];
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
                            {
                                let b_drop = drop.borrow();
                                for i in 0..self.blocksize {
                                    self.data[block.begin_at+i] = b_drop.data[i];
                                }
                            }
                            block.is_known = true;
                            self.unknown_chunks -= 1;

                            while block.edges.len() > 0 {
                                let edge = block.edges.pop().unwrap();
                                let mut m_edge = edge.borrow_mut();

                                if m_edge.edges_idx.len() == 1 {
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

    /// Catches a Droplet
    /// When it is possible to reconstruct a set, the bytes are returned
    pub fn catch(&mut self, drop: Droplet) -> CatchResult {
        self.cnt_received_drops +=1;

        let sample = get_sample_from_rng_by_seed(drop.seed, self.number_of_chunks, drop.degree);
        let rxdrop = RxDroplet {edges_idx: sample, data: drop.data};
        self.process_droplet(rxdrop);
        let stats = Statistics {
            cnt_droplets: self.cnt_received_drops,
            cnt_chunks: self.number_of_chunks,
            overhead: self.cnt_received_drops as f32 * 100.0 / self.number_of_chunks as f32,
            unknown_chunks: self.unknown_chunks
        };

        if self.unknown_chunks == 0 {
            let mut result = Vec::with_capacity(self.total_length);
            for i in 0..self.total_length { //TODO: we should be able to do that without copying
                result.push(self.data[i]);
            }
            CatchResult::Finished(result, stats)
        }
        else {
            CatchResult::Missing(stats)
        }
    }
}
