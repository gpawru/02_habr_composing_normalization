#[macro_use]
extern crate lazy_static;

use std::fs::File;

mod encode;
mod output;
mod tables;

fn main()
{
    output::write(true, &mut File::create("./../data/nfc.rs.txt").unwrap());
    output::write(false, &mut File::create("./../data/nfkc.rs.txt").unwrap());
}
