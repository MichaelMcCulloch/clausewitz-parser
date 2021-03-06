use std::{
    fs::{self, File},
    io::{BufRead, Read},
    os::unix::prelude::MetadataExt,
    time::Instant,
};

use clausewitz_parser::{key_value, par_root};
use memmap::Mmap;
use nom::InputTake;

fn main() {
    let filename = "/home/michael/Desktop/gamestate";

    let mut file = File::open(filename).expect("File not found");
    let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {:?}", file)) };

    let str = String::from_utf8_lossy(&mmap[..]);

    let pos = str.find("country={").unwrap();
    let rep = str.split_at(pos).1;
    // let replace = str.replace("\n}\n", "\n}\n#");

    let start = Instant::now();

    let _result = key_value(&rep).unwrap().1;

    let end = start.elapsed();

    let size_in_bytes = file.metadata().unwrap().size();
    let speed = (size_in_bytes as u128 / end.as_millis()) * 1000;

    println!(
        "{:?}MB/s, took {} ms.",
        speed as f32 / 1000000 as f32,
        end.as_millis()
    );

    // println!("{}", _result.1);
}
