use std::{
    fs::File,
    ops::{Add, Div},
    time::Instant,
};

use clausewitz_parser::cheat_root;
use memmap::Mmap;

fn main() {
    let filename = "C:\\Users\\micha\\Documents\\Paradox Interactive\\Stellaris\\save games\\mptebridhomolog2_-66053362\\gamestate";

    let file = File::open(filename).expect("File not found");
    let mmap =
        unsafe { Mmap::map(&file).unwrap_or_else(|_| panic!("Error mapping file {:?}", file)) };

    let str = String::from_utf8_lossy(&mmap[..]);

    let size_in_bytes = file.metadata().unwrap().len();

    let mut times = vec![];
    let count = 10;
    for _ in 0..count {
        let start_parse = Instant::now();
        let _ = cheat_root(
            &str,
            vec![
                "version",
                "player",
                "country",
                "fleet",
                "ships",
                "ship_design",
            ],
        );

        let end_parse = start_parse.elapsed();

        times.push(end_parse);
    }
    let avg = times
        .into_iter()
        .reduce(|a, b| a.add(b))
        .unwrap()
        .div(count);
    println!(
        "{:?}MB/s, took {} ms.",
        ((size_in_bytes as u128 / avg.as_millis()) * 1000) as f32 / 1000000_f32,
        avg.as_millis()
    );

    // let (s, v) = cheat_root(&str, vec!["country"]).unwrap();

    // println!("{:?}", v)
}
