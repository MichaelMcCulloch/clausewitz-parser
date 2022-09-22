use std::{
    fs::File,
    os::unix::prelude::MetadataExt,
    thread,
    time::{Duration, Instant},
};

use clausewitz_parser::{
    root,
    skim::{isp::ISP, search_document},
    skip_par_root, ClausewitzValue,
};
use memmap::Mmap;

fn main() {
    let filename = "/home/michael/Dev/Stellarust/clausewitz-parser/production_data/3.4.5.95132/hotjoin_2290.03.05/gamestate";

    let file = File::open(filename).expect("File not found");
    let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {:?}", file)) };

    let str = String::from_utf8_lossy(&mmap[..]);
    let input = ISP::create(
        &str,
        "country.0.budget.current_month.income.country_base.energy",
    );
    // let input = InputSearchPair::create(text, "flag.icon");//fails
    let size_in_bytes = file.metadata().unwrap().size();

    let start_direct = Instant::now();

    let result = search_document(input);

    let end_direct = start_direct.elapsed();
    let speed = (size_in_bytes as u128 / end_direct.as_millis()) * 1000;
    println!(
        "{:?}MB/s, took {} ms.",
        speed as f32 / 1000000 as f32,
        end_direct.as_millis()
    );
    // thread::sleep(Duration::from_secs(10));

    drop(result);

    let start_parse = Instant::now();
    let result = skip_par_root(&str, "\n}\n");

    let end_parse = start_parse.elapsed();

    let speed = (size_in_bytes as u128 / end_parse.as_millis()) * 1000;
    println!(
        "{:?}MB/s, took {} ms.",
        speed as f32 / 1000000 as f32,
        end_parse.as_millis()
    );
    // thread::sleep(Duration::from_secs(10));
    drop(result);
    // assert!(result.is_ok());
    // let succ = result.unwrap();

    // let v = succ.1.get_at_path("country").unwrap();

    // match v {
    //     clausewitz_parser::Val::Dict(_) => println!("Dict"),
    //     clausewitz_parser::Val::NumberedDict(_, _) => println!("NumberedDict"),
    //     clausewitz_parser::Val::Array(_) => println!("Array"),
    //     clausewitz_parser::Val::Set(_) => println!("Set"),
    //     clausewitz_parser::Val::StringLiteral(_) => println!("StringLiteral"),
    //     clausewitz_parser::Val::Date(_) => println!("Date"),
    //     clausewitz_parser::Val::Decimal(_) => println!("Decimal"),
    //     clausewitz_parser::Val::Integer(_) => println!("Integer"),
    //     clausewitz_parser::Val::Identifier(_) => println!("Identifier"),
    // }

    // println!("{}", _result.1);
}
