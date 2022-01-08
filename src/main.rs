use std::{fs::File, os::unix::prelude::MetadataExt, time::Instant};

use clausewitz_parser::root;
use memmap::Mmap;

fn main() {
    let filename = "/path/to/unzipped/gamestate/or/meta";

    let file = File::open(filename).expect("File not found");
    let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {:?}", file)) };

    let str = std::str::from_utf8(&mmap[..]).unwrap();

    let start = Instant::now();
    let _result = root(str);

    let end = start.elapsed();

    let size_in_bytes = file.metadata().unwrap().size();
    let speed = (size_in_bytes as u128 / end.as_millis()) * 1000;

    println!(
        "{:?}MB/s, took {} ms.",
        speed as f32 / 1000000 as f32,
        end.as_millis()
    );

    // println!("{}", result.unwrap().1);
}
