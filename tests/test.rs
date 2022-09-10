#[cfg(test)]
mod file_test {
    use std::fs::{self, File};

    use clausewitz_parser::root;
    use memmap::Mmap;
    use nom::InputTake;

    #[test]
    fn meta() {
        let text = fs::read_to_string(
            "/home/michael/Dev/Stellarust/clausewitz-parser/production_data/3.4.5.95132/2230.12.01/meta",
        )
        .unwrap();
        let result = root(&text);

        assert!(result.is_ok());
    }

    #[test]
    fn gamestate_memmap_par_root__for_epic_files() {
        let filename =
            "/home/michael/Dev/Stellarust/clausewitz-parser/production_data/3.4.5.95132/2290.03.05/gamestate";
        let file = File::open(filename).expect("File not found");

        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {:?}", file)) };

        let str = String::from_utf8_lossy(&mmap[..]);

        let result = root(&str);

        // assert!(result.is_ok());
        let r = result.err().unwrap();
        println!("{}", &r.to_string()[0..100]);
        // assert!(r.is_empty())
    }
}
