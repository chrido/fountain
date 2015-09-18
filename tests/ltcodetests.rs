extern crate rand;
extern crate fountaincode;

use self::fountaincode::ltcode::{Encoder, EncoderType, Decoder};
use self::fountaincode::ltcode::CatchResult::*;

use rand::{thread_rng, Rng};


fn encode_decode_random(total_len: usize, chunk_len: usize) {
    let s:String = thread_rng().gen_ascii_chars().take(total_len).collect();
    let buf = s.into_bytes();
    let len = buf.len();
    let to_compare = buf.clone();

    let enc = Encoder::new(buf, chunk_len, EncoderType::Random);
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

fn encode_decode_systematic(total_len: usize, chunk_len: usize) {
    let s:String = thread_rng().gen_ascii_chars().take(total_len).collect();
    let buf = s.into_bytes();
    let len = buf.len();
    let to_compare = buf.clone();

    let enc = Encoder::new(buf, chunk_len, EncoderType::Systematic);
    let mut dec = Decoder::new(len, chunk_len);

    let mut cnt_drops = 0;

    for drop in enc {
        cnt_drops += 1;
        match dec.catch(drop) {
            Missing(stats) => {
                //a systematic encoder and no loss on channel should only need k symbols
                assert_eq!(stats.cnt_chunks-stats.unknown_chunks, cnt_drops)
            }
            Finished(data, stats) => {
                assert_eq!(to_compare.len(), data.len());
                assert_eq!(stats.cnt_chunks, cnt_drops);
                for i in 0..len {
                    assert_eq!(to_compare[i], data[i]);
                }
                return
            }
        }
    }
}

fn encode_decode_systematic_with_loss(total_len: usize, chunk_len: usize, loss: f32) {
    let s:String = thread_rng().gen_ascii_chars().take(total_len).collect();
    let buf = s.into_bytes();
    let len = buf.len();
    let to_compare = buf.clone();

    let enc = Encoder::new(buf, chunk_len, EncoderType::Systematic);
    let mut dec = Decoder::new(len, chunk_len);

    let mut cnt_drops = 0;
    let mut loss_rng = thread_rng();

    for drop in enc {
        cnt_drops += 1;
        if loss_rng.next_f32() > loss {
            match dec.catch(drop) {
                Missing(stats) => {
                    //a systematic encoder and no loss on channel should only need k symbols
                    //assert_eq!(stats.cnt_chunks-stats.unknown_chunks, cnt_drops)
                }
                Finished(data, stats) => {
                    assert_eq!(to_compare.len(), data.len());
                    for i in 0..len {
                        assert_eq!(to_compare[i], data[i]);
                    }
                    return
                }
            }
        }
    }
}



#[test]
fn test_encode_decode_simple() {
    encode_decode_random(1_024, 512);
}

#[test]
fn encode_decode_with_uneven_sizes_random() {
    for size in 1000..1100 {
        for chunk in 100..130 {
            encode_decode_random(size, chunk);
        }
    }
}

#[test]
fn small_test_systematic_encoder() {
    encode_decode_systematic(1300, 128);
}

#[test]
fn combinations_encode_decode_with_uneven_sizes_systematic() {
    for size in 1000..1100 {
        for chunk in 100..130 {
            encode_decode_systematic(size, chunk);
        }
    }
}

#[test]
fn small_encode_decode_with_loss_begin_with_systematic() {
    encode_decode_systematic_with_loss(8, 2, 0.1);
}

#[test]
fn combinations_encode_decode_with_loss_begin_with_systematic() {
    for size in 1000..1100 {
        for chunk in 100..130 {
            for loss in vec![0.1, 0.3, 0.5, 0.9] {
                 encode_decode_systematic_with_loss(size, chunk, loss);
            }
        }
    }
}
