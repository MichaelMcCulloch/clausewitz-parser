#[cfg(test)]
mod file_test {
    use std::fs::{self, File};

    use clausewitz_parser::{par_root, root};
    use memmap::Mmap;

    #[test]
    fn meta() {
        let text = fs::read_to_string("/home/michael/Dev/stellarust/res/test_data/campaign_raw/unitednationsofearth_-15512622/autosave_2200.02.01/meta").unwrap();
        let result = root(&text);

        assert!(result.is_ok());
    }

    #[test]
    fn gamestate() {
        let text = fs::read_to_string("/home/michael/Dev/stellarust/res/test_data/campaign_raw/unitednationsofearth_-15512622/autosave_2200.02.01/gamestate").unwrap();

        let result = root(&text);

        assert!(result.is_ok());
    }

    #[test]
    fn gamestate_memmap_par_root__for_epic_files() {
        let filename = "/home/michael/Dev/stellarust/res/test_data/campaign_raw/unitednationsofearth_-15512622/autosave_2200.02.01/gamestate";
        let file = File::open(filename).expect("File not found");

        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {:?}", file)) };

        let str = std::str::from_utf8(&mmap[..]).unwrap();
        let prepared_input = str.replace("\n}\n", "\n}\n#");

        let result = par_root(&prepared_input);

        assert!(result.is_ok());
    }
}
