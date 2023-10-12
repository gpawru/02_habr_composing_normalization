use crate::output::format::format_num_vec;
use std::fs::File;

mod encode;
mod output;
mod pairs;
mod tables;

fn main()
{
    // pairs::nfc();

    output::write(
        true,
        &mut File::create("./../data/nfc.rs.txt").unwrap(),
        &mut File::create("./../data/nfc.stats.txt").unwrap(),
    );
    output::write(
        false,
        &mut File::create("./../data/nfkc.rs.txt").unwrap(),
        &mut File::create("./../data/nfkc.stats.txt").unwrap(),
    );
}
