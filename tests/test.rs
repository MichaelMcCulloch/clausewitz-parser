#[cfg(test)]
mod test {
    use std::fs::{self, File};

    use chrono::NaiveDate;
    use clausewitz_parser::{par_root, root, Val};
    use memmap::Mmap;

    #[test]
    fn meta() {
        let text = fs::read_to_string("/home/michael/Dev/stellarust/res/test_data/campaign_raw/unitednationsofearth_-15512622/autosave_2200.02.01/meta").unwrap();
        let result = root(text.as_str());

        assert!(result.is_ok());
    }

    #[test]
    fn gamestate() {
        let text = fs::read_to_string("/home/michael/Dev/stellarust/res/test_data/campaign_raw/unitednationsofearth_-15512622/autosave_2200.02.01/gamestate").unwrap();

        let result = root(text.as_str());

        assert!(result.is_ok());
    }

    #[test]
    fn gamestate_memmap_par_root__for_epic_files() {
        let filename = "/home/michael/Dev/stellarust/res/test_data/campaign_raw/unitednationsofearth_-15512622/autosave_2200.02.01/gamestate";
        let file = File::open(filename).expect("File not found");

        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {:?}", file)) };

        let str = std::str::from_utf8(&mmap[..]).unwrap();
        let prepared_input = str.replace("\n}\n", "\n}\n#");

        let result = par_root(prepared_input.as_str());

        assert!(result.is_ok());
    }
    #[test]
    fn format_integer() {
        let _str = format!("{}", Val::Integer(0));
    }

    #[test]
    fn format_decimal() {
        let _str = format!("{}", Val::Decimal(0.0));
    }

    #[test]
    fn format_identifier() {
        let _str = format!("{}", Val::Identifier("identifier"));
    }

    #[test]
    fn format_string_literal() {
        let _str = format!("{}", Val::StringLiteral("String Litteral"));
    }

    #[test]
    fn format_date() {
        let _str = format!("{}", Val::Date(NaiveDate::from_ymd(2021, 1, 1)));
    }

    #[test]
    fn format_set() {
        let _str = format!(
            "{}",
            Val::Set(vec![Val::Integer(0), Val::Set(vec![Val::Integer(0)])])
        );
    }

    #[test]
    fn format_dict() {
        let _str = format!(
            "{}",
            Val::Dict(vec![
                ("key", Val::Integer(0)),
                ("dict", Val::Dict(vec![("key", Val::Integer(0))]))
            ])
        );
    }

    #[test]
    fn format_dict2() {
        let _str = format!("{}", Val::Dict(vec![("key", Val::Integer(0)),]));
    }

    #[test]
    fn format_NumberedDict() {
        let _str = format!(
            "{}",
            Val::NumberedDict(
                0,
                vec![
                    ("key", Val::Integer(0)),
                    (
                        "NumberedDict",
                        Val::NumberedDict(1, vec![("key", Val::Integer(0))])
                    )
                ]
            )
        );
    }

    #[test]
    fn format_NumberedDict2() {
        let _str = format!(
            "{}",
            Val::NumberedDict(-234, vec![("key", Val::Integer(0)),])
        );
    }
}
