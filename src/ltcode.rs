use std::vec::Vec;
use std::cell::RefCell;
use std::rc::Rc;
use std::cmp;
use rand::{Rng, sample, StdRng, SeedableRng};

use soliton::IdealSoliton;

#[derive(Clone, Debug)]
pub enum EncoderType {
    /// The first k symbols of a systematic Encoder correspond to the first k source symbols
    /// In case there is no loss, no repair needed. After the first k symbols are sent, it continous
    /// like in the Random case.
    Systematic,
    /// Begins immediately with random encoding. This may be a better choice when used with high-loss channels.
    Random
}

/// Encoder for Luby transform codes
pub struct Encoder {
    data: Vec<u8>,
    len: usize,
    blocksize: usize,
    rng: StdRng,
    cnt_blocks: usize,
    sol: IdealSoliton,
    cnt: usize,
    encodertype: EncoderType
}

#[derive(Debug)]
enum DropType {
    ///First is seed, second degree
    Seeded(usize, usize),
    ///Just a list of edges
    Edges(Vec<usize>)
}

/// A Droplet is created by the Encoder.
#[derive(Debug)]
pub struct Droplet {
    /// The droptype can be based on seed or a list of edges
    droptype: DropType,
    /// The payload of the Droplet
    data: Vec<u8>
}

impl Droplet {
    fn new(droptype: DropType, data: Vec<u8>) -> Droplet {
        Droplet {droptype: droptype, data: data}
    }
}

impl Encoder {
    /// Constructs a new encoder for Luby transform codes.
    /// In case you send the packages over UDP, the blocksize
    /// should be the MTU size.
    ///
    /// There are two versions of the 'Encoder', Systematic and Random.
    /// The Systematic encoder first produces a set of the source symbols. After each
    /// symbol is sent once, it switches to Random.
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
    ///     use fountaincode::ltcode::{Encoder, EncoderType};
    ///     use self::rand::{thread_rng, Rng};
    ///
    ///     let s:String = thread_rng().gen_ascii_chars().take(1_024).collect();
    ///     let buf = s.into_bytes();
    ///
    ///     let mut enc = Encoder::new(buf, 64, EncoderType::Random);
    ///
    ///     for i in 1..10 {
    ///         println!("droplet {:?}: {:?}", i, enc.next());
    ///     }
    /// }
    /// ```
    pub fn new(data: Vec<u8>, blocksize: usize, encodertype: EncoderType) -> Encoder {
        let mut rng = StdRng::new().unwrap();

        let len = data.len();
        let cnt_blocks = ((len as f32)/blocksize as f32).ceil() as usize;
        let sol = IdealSoliton::new(cnt_blocks, rng.gen::<usize>());
        Encoder{data: data, len: len, blocksize: blocksize, rng: rng, cnt_blocks: cnt_blocks, sol: sol, cnt: 0, encodertype: encodertype}
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
        let drop = match self.encodertype {
            EncoderType::Random => {
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
                Some(Droplet::new(DropType::Seeded(seed, degree), r))
            },
            EncoderType::Systematic => {
                let begin = self.cnt * self.blocksize;
                let end = cmp::min((self.cnt + 1) * self.blocksize, self.len);
                let mut r:Vec<u8> = vec![0; self.blocksize];

                let mut j = 0;
                for i in begin..end {
                    r[j] = self.data[i];
                    j+=1;
                }
                if (self.cnt + 2)  > self.cnt_blocks {
                    self.encodertype = EncoderType::Random;
                }
                Some(Droplet::new(DropType::Edges(vec![self.cnt]), r))
            }
        };

        self.cnt += 1;
        drop
    }
}

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
    /// Creates a new Decoder for LT codes
    ///
    /// # Example
    ///
    /// ```
    /// extern crate rand;
    /// extern crate fountaincode;
    ///
    /// fn main() {
    ///     use fountaincode::ltcode::{Encoder, EncoderType, Decoder};
    ///     use fountaincode::ltcode::CatchResult::*;
    ///     use self::rand::{thread_rng, Rng};
    ///
    ///     let s:String = thread_rng().gen_ascii_chars().take(1_024).collect();
    ///     let buf = s.into_bytes();
    ///     let to_compare = buf.clone();
    ///     let length = buf.len();
    ///
    ///     let mut enc = Encoder::new(buf, 64, EncoderType::Random);
    ///     let mut dec = Decoder::new(length, 64);
    ///
    ///     for drop in enc {
    ///         match dec.catch(drop) {
    ///             Missing(stats) => {
    ///                 println!("Missing blocks {:?}", stats);
    ///             }
    ///             Finished(data, stats) => {
    ///                 for i in 0..length {
    ///                     assert_eq!(to_compare[i], data[i]);
    ///                 }
    ///                 println!("Finished, stas: {:?}", stats);
    ///                 //write data to disk??
    ///                 return
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn new(len: usize, blocksize: usize) -> Decoder {
        let number_of_chunks = ((len as f32)/blocksize as f32).ceil() as usize;
        let data:Vec<u8> = vec![0; number_of_chunks * blocksize];
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
                    //TODO: Maybe add shortcut for the first wave of systematic codes, reduce overhead

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
        let sample:Vec<usize> = match drop.droptype {
            DropType::Seeded(seed, degree) => {
                get_sample_from_rng_by_seed(seed, self.number_of_chunks, degree)
            }
            DropType::Edges(edges) => {edges}
        };

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
