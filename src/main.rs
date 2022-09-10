use std::{fs::File, os::unix::prelude::MetadataExt, time::Instant};

use clausewitz_parser::{root, ClausewitzValue};
use memmap::Mmap;

fn main() {
    let filename = "/home/michael/Dev/Stellarust/clausewitz-parser/production_data/3.4.5.95132/hotjoin_2290.03.05/gamestate";

    let file = File::open(filename).expect("File not found");
    let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {:?}", file)) };

    let str = String::from_utf8_lossy(&mmap[..]);

    let start = Instant::now();

    let result = root(&str);

    let end = start.elapsed();

    let size_in_bytes = file.metadata().unwrap().size();
    // let speed = (size_in_bytes as u128 / end.as_millis()) * 1000;
    assert!(result.is_ok());
    let succ = result.unwrap();

    let v = succ.1.get_at_path("country").unwrap();

    match v {
        clausewitz_parser::Val::Dict(_) => println!("Dict"),
        clausewitz_parser::Val::NumberedDict(_, _) => println!("NumberedDict"),
        clausewitz_parser::Val::Array(_) => println!("Array"),
        clausewitz_parser::Val::Set(_) => println!("Set"),
        clausewitz_parser::Val::StringLiteral(_) => println!("StringLiteral"),
        clausewitz_parser::Val::Date(_) => println!("Date"),
        clausewitz_parser::Val::Decimal(_) => println!("Decimal"),
        clausewitz_parser::Val::Integer(_) => println!("Integer"),
        clausewitz_parser::Val::Identifier(_) => println!("Identifier"),
    }

    // println!(
    //     "{:?}MB/s, took {} ms.",
    //     speed as f32 / 1000000 as f32,
    //     end.as_millis()
    // );

    // println!("{}", _result.1);
}
