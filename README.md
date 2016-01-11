# Fountain codes

[![Build status](https://travis-ci.org/chrido/fountain.svg?branch=master)](https://travis-ci.org/chrido/fountain)
[![Crates](http://meritbadge.herokuapp.com/fountaincode)](https://crates.io/crates/fountaincode)

The library implements the Luby Transform Code in [Rust](https://www.rust-lang.org/).
For more information have a look at [Wikipedia](https://en.wikipedia.org/wiki/Luby_transform_code) or the paper [LT codes](http://dx.doi.org/10.1109/SFCS.2002.1181950) on IEEE Xplore.

In future I might add [RaptorQ](http://tools.ietf.org/html/rfc6330) or [Online](http://pdos.csail.mit.edu/~petar/papers/maymounkov-online.pdf) [code](http://www.scs.stanford.edu/~dm/home/papers/maymounkov:rateless.pdf).

## Dependencies
`rand`

## Usage
Add `fountaincode` as a dependency in `Cargo.toml`

```toml
[dependencies]
fountaincode = "*"
```

## Example

```rust
let mut buf = Vec::new();
let mut f = File::open("testfile.bin").unwrap();
try!(f.read_to_end(&mut buf));

let length = buf.len();
let buf_org = buf.clone();

//create an Encoder, and set the length of the chunks.
//In case UDP is used you may want to stay below the MTU size
let enc = Encoder::new(buf, 1024);

//create a Decoder
let mut dec = Decoder::new(length, 1024);

//Encoder is exposed as Iterator
//In practice you may want to send over a Binary Error Channel, e.g., UDP
for drop in enc {

    //Decoder catches droplets
    //In practice you may want to listen on a UDP port for packages
    match dec.catch(drop) {
        Missing(stats) => {
            trace!("{:?} chunks are unknown", cnt.unknown_chunks);
        }
        Finished(data, stats) => {
            done = true;
            println!("finished! {:?}", stats);
            for i in 0..length {
                assert!(buf_org[i], data[i]);
            }
            println!("and the data is correct!");
        }
    }
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
