#[cfg(test)]
mod file_test {
    use std::fs::{self, File};

    use clausewitz_parser::root;
    use memmap::Mmap;

    #[test]
    fn meta() {
        let text = fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/production_data/2337.02.02-testing/meta"
        ))
        .unwrap();
        let result = root(&text);

        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(
            r.0.is_empty(),
            "Unparsed remainder: {:?}",
            &r.0[..r.0.len().min(200)]
        );
    }

    #[test]
    fn gamestate_memmap_root_for_epic_files() {
        let filename = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/production_data/2337.02.02-testing/gamestate"
        );
        let file = File::open(filename).expect("File not found");

        let mmap =
            unsafe { Mmap::map(&file).unwrap_or_else(|_| panic!("Error mapping file {:?}", file)) };

        let str = String::from_utf8_lossy(&mmap[..]);

        let result = root(&str);

        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let r = result.unwrap();
        assert!(
            r.0.is_empty(),
            "Unparsed remainder: {:?}",
            &r.0[..r.0.len().min(200)]
        );
    }
}
