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
    test_fountain().unwrap()
}

fn test_fountain() -> Result<(), Error> {
    let mut buf = Vec::new();
    let mut f = File::open("testfile.bin").unwrap();
    try!(f.read_to_end(&mut buf));
    let length = buf.len();

    let mut enc = Encoder::new(buf, 1024);
    let mut dec = Decoder::new(length, 1024);

    let mut done = false;

    while !done {
        let drop = enc.next().unwrap();
        match dec.catch(drop) {
            Missing(cnt) => {
                trace!("missing: {:?}", cnt);
            }
            Finished(data) => {
                done = true;
                debug!("finished!");
            }
        }
    }

    Ok(())
}
