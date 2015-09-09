extern crate rand;

#[macro_use]
extern crate log;
extern crate env_logger;

pub mod soliton;
pub mod ltcode;
use ltcode::Encoder;
use ltcode::Decoder;
use ltcode::CatchResult::*;

use std::io::Error;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    env_logger::init().unwrap();
    for _ in 0..1 {
        test_fountain().unwrap();
    }
}

fn test_fountain() -> Result<(), Error> {
    let mut buf = Vec::new();
    let mut f = File::open("testfile.bin").unwrap();
    try!(f.read_to_end(&mut buf));
    let length = buf.len();
    debug!("len: {:?}", length);
    let buf_org = buf.clone();

    let mut enc = Encoder::new(buf, 1024);
    let mut dec = Decoder::new(length, 1024);

    let mut done = false;

    while !done {
        let drop = enc.next().unwrap();
        match dec.catch(drop) {
            Missing(cnt) => {
                trace!("missing: {:?}", cnt);
            }
            Finished(data, stats) => {
                done = true;
                debug!("finished! {:?}", stats);
                for i in 0..length {
                    if (buf_org[i] != data[i]){
                        debug!("i: {:?}", i);
                    }
                    assert_eq!(buf_org[i], data[i]);
                }
                debug!("result correct");
            }
        }
    }

    Ok(())
}
