#[cfg(test)]
mod test {
    use std::fs::{self, File};

    use clausewitz_parser::{root, Val};
    use memmap::Mmap;

    use time::{Date, Month};

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
    fn gamestate_memmap__for_epic_files() {
        let filename = "/home/michael/Dev/stellarust/res/test_data/campaign_raw/unitednationsofearth_-15512622/autosave_2200.02.01/gamestate";
        let file = File::open(filename).expect("File not found");

        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {:?}", file)) };

        let str = std::str::from_utf8(&mmap[..]).unwrap();
        let prepared_input = str.replace("\n}\n", "\n}\n#");

        let result = root(prepared_input.as_str());

        // println!("{:#?}", result);
        println!("{}", result.unwrap().1);

        // assert!(result.is_ok());
    }
    #[test]
    fn format_integer() {
        println!("{}", Val::Integer(0));
    }

    #[test]
    fn format_decimal() {
        println!("{}", Val::Decimal(0.0));
    }

    #[test]
    fn format_identifier() {
        println!("{}", Val::Identifier("identifier"));
    }

    #[test]
    fn format_string_literal() {
        println!("{}", Val::StringLiteral("String Litteral"));
    }

    #[test]
    fn format_date() {
        println!(
            "{}",
            Val::Date(Date::from_calendar_date(2021, Month::January, 1).unwrap())
        );
    }

    #[test]
    fn format_set() {
        println!(
            "{}",
            Val::Set(vec![Val::Integer(0), Val::Set(vec![Val::Integer(0)])])
        );
    }

    #[test]
    fn format_dict() {
        println!(
            "{}",
            Val::Dict(vec![
                ("key", Val::Integer(0)),
                ("dict", Val::Dict(vec![("key", Val::Integer(0))]))
            ])
        );
    }

    #[test]
    fn format_dict2() {
        println!("{}", Val::Dict(vec![("key", Val::Integer(0)),]));
    }

    #[test]
    fn format_NumberedDict() {
        println!(
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
        println!(
            "{}",
            Val::NumberedDict(-234, vec![("key", Val::Integer(0)),])
        );
    }
}
