use crate::output::format::format_num_vec;
use std::fs::File;

mod encode;
mod output;
mod tables;

fn main()
{
    output::write(
        true,
        &mut File::create("./../data/nfd.rs.txt").unwrap(),
        &mut File::create("./../data/nfd.stats.txt").unwrap(),
    );
    output::write(
        false,
        &mut File::create("./../data/nfkd.rs.txt").unwrap(),
        &mut File::create("./../data/nfkd.stats.txt").unwrap(),
    );
}
