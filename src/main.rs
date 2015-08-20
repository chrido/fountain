extern crate rand;

pub mod lib;
use lib::Soliton;

fn main() {
    let sol = Soliton::new(10, 18);
    let a = sol.take(100).collect::<Vec<_>>();
    println!("Soliton {:?}", a);
}
