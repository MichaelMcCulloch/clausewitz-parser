#[cfg(test)]
mod file_test {
    use std::fs::{self, File};

    use clausewitz_parser::root;
    use memmap::Mmap;

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

        let str = std::str::from_utf8(&mmap[..]).unwrap();

        let result = root(&str);

        assert!(result.is_ok());
        assert!(result.unwrap().0.is_empty())
    }
}
