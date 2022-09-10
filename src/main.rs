use std::{fs::File, os::unix::prelude::MetadataExt, time::Instant};

use clausewitz_parser::root;
use memmap::Mmap;

fn main() {
    let filename = "/home/michael/Dev/Stellarust/clausewitz-parser/production_data/3.4.5.95132/2213.12.13/gamestate";

    let file = File::open(filename).expect("File not found");
    let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {:?}", file)) };

    let str = String::from_utf8_lossy(&mmap[..]);

    let start = Instant::now();

    let result = root(&str);

    let end = start.elapsed();

    let size_in_bytes = file.metadata().unwrap().size();
    let speed = (size_in_bytes as u128 / end.as_millis()) * 1000;
    assert!(result.is_ok());
    assert!(result.unwrap().0.is_empty());
    println!(
        "{:?}MB/s, took {} ms.",
        speed as f32 / 1000000 as f32,
        end.as_millis()
    );

    // println!("{}", _result.1);
}
