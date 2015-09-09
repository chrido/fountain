# Fountain codes

The library implements currently the Luby Transform Code in [Rust](https://www.rust-lang.org/).
For more information have a look at [Wikipedia](https://en.wikipedia.org/wiki/Luby_transform_code) or the paper [LT codes](http://dx.doi.org/10.1109/SFCS.2002.1181950) on IEEE Xplore.

In future I might add [RaptorQ](http://tools.ietf.org/html/rfc6330)

## Dependencies
rand

## Usage
Add `fountaincode` as a dependency in `Cargo.toml`

```toml
[dependencies]
fountaincode = "0.0.1"
```

## Example

```rust
let mut buf = Vec::new();
let mut f = File::open("testfile.bin").unwrap();
try!(f.read_to_end(&mut buf));

let length = buf.len();
let buf_org = buf.clone();

//create an Encoder, and set the length of the chunks.
//In case UDP is used you may want to stay below under the MTU size
let enc = Encoder::new(buf, 1024);

//create a Decoder
let mut dec = Decoder::new(length, 1024);

for drop in enc { //Encoder is exposed as Iterator, in practice you may want to send over UDP
    //Decoder catches droplets, in practice you may want to listen on a UDP port for packages
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

        }
    }
}
```
