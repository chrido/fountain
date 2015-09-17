extern crate rand;
extern crate fountaincode;

use self::fountaincode::ltcode::{Encoder, Decoder};
use self::fountaincode::ltcode::CatchResult::*;

use rand::{thread_rng, Rng};


fn encode_decode(total_len: usize, chunk_len: usize) {
    let s:String = thread_rng().gen_ascii_chars().take(total_len).collect();
    let buf = s.into_bytes();
    let len = buf.len();
    let to_compare = buf.clone();

    let enc = Encoder::new(buf, chunk_len);
    let mut dec = Decoder::new(len, chunk_len);

    for drop in enc {
        match dec.catch(drop) {
            Missing(stats) => {
                println!("Missing blocks {:?}", stats);
            }
            Finished(data, stats) => {
                assert_eq!(to_compare.len(), data.len());
                for i in 0..len {
                    println!("stats: {:?}", stats);
                    assert_eq!(to_compare[i], data[i]);
                }
                println!("Finished, stas: {:?}", stats);
                return
            }
        }
    }
}


#[test]
fn test_decode() {
    encode_decode(1_024, 512);
}

#[test]
fn test_with_different_sizes() {
    for size in 1000..1100 {
        for chunk in 100..130 {
            encode_decode(size, chunk);
        }
    }
}
