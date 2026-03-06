use nom::combinator::map;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;

use super::{bracketed::hash_map, space::opt_space, val::Val, Res};
#[inline(always)]
pub fn root<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    let (input, val) = map(hash_map, Val::Dict)(input)?;
    let (input, _) = opt_space(input)?;
    Ok((input, val))
}

#[inline(always)]
pub fn cheat_root<'a>(input: &'a str, keys: Vec<&str>) -> Res<&'a str, Val<'a>> {
    let mut last = 0;
    let mut indices: Vec<&str> = vec![];
    // "\n\w+=.*\n" may be a better way to split up the file by top-level keys
    let regex = Regex::new(r"\n\w+=.*|^version=.*").expect("invalid_regex");
    for mat in regex.find_iter(input) {
        if mat.start() == 0 {
            continue;
        }
        let start = mat.start() + 1;
        if start != last {
            indices.push(&input[last..start])
        }
        last = start;
    }
    if last < input.len() {
        indices.push(&input[last..]);
    }
    let res = Val::Dict(
        indices
            .iter()
            .filter(|block| {
                keys.iter()
                    .any(|k| block.starts_with(format!("{}=", k).as_str()))
            })
            .collect::<Vec<_>>()
            .par_iter()
            .filter_map(|string| match root(string) {
                Ok((_, Val::Dict(dict))) => Some(dict),
                Ok(_) => None,
                Err(_) => None,
            })
            .flat_map(|v| v)
            .collect(),
    );
    Ok(("", res))
}

#[cfg(test)]
mod tests {
    use crate::{clausewitz::tests::helper::assert_result_ok, key_value};
    #[test]
    fn root_key_identifier_pairs_ok() {
        let text = r###"dict={
    alpha=a
    beta=b
    cthulhu=ilhjok
}
dict2={
    charlie=a
    delta=b
    zoo=ilhjok
}"###;

        let result = cheat_root(text, vec!["version", "player", "country", "fleet", "ships"]);

        assert_result_ok(result);
    }
    use super::*;
    #[test]
    fn basics() {
        let text = r###"vers_ion0="Herbert v3.2.2"
            version_control_revision=83287
            date="2200.05.01"
            date="0.05.01"
            float=-0.123939887"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn set_numbers_same_line() {
        let text = r###"set_of_numbers={
    40 41
}"###;

        let result = root(text);
        assert_result_ok(result);
    }
    #[test]
    fn space_not_new_line() {
        let text = r###"modules={
                0=shipyard				1=trading_hub			}"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn kv_pair_starts_with_number() {
        let text = r###"flags={
            3_year_owner_change_flag={
                flag_date=63568248
                flag_days=293
            }
        }"###;
        let result = key_value(text);
        println!("{:?}", result);

        assert_result_ok(result);
    }

    #[test]
    fn intel_numbered_dicts() {
        let text = r###"intel={
                                    {
                                        14 {
                                            intel=0
                                            stale_intel={
                                            }
                                        }
                                    }
                                    {
                                        19 {
                                            intel=0
                                            stale_intel={
                                            }
                                        }
                                    }
                                }"###;
        let result = root(text);

        assert_result_ok(result);
    }

    #[test]
    fn dict_of_dicts() {
        let text = r###"dict_of_dicts={
                icon={
                    category="human"
                    file="flag_human_9.dds"
                }
                background={
                    category="backgrounds"
                    file="00_solid.dds"
                }
                colors={
                    "blue"
                    "black"
                    "null"
                    "null"
                }
            }"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn quoted_key_ok() {
        let text = r###""The name Of A Ship"=0"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn empty_set_set() {
        let text = r###"empty_set={}"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn root_set_of_strings_accepted() {
        let text = r###"set_of_strings={
                "Ancient Relics Story Pack"
                "Anniversary Portraits"
                "Apocalypse"
            }"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn array_of_arrays() {
        let text = r###"array_of_arrays={
                0={
                    0="a"
                }
                1={
                    0="one"
                }
                2={
                    0="two"
                }
            }"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn identifier_with_underscore() {
        let text = r###"identifier=identi_fire"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn dict_key_identifier_pairs_ok() {
        let text = r###"dict={
                alpha=a
                beta=b
                cthulhu=ilhjok
            }"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn newline_between_equals_and_brace() {
        let text = "required_dlcs=\n{\n\t\"Ancient Relics Story Pack\"\n\t\"Apocalypse\"\n}";

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn none_as_value() {
        let text = "key=none";

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn none_in_dict() {
        let text = "entries={\n\t218104267=none\n\t83886542=none\n}";

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn yes_no_values() {
        let text = "date_distortion=no\ncheated_on_save=no\nrandomized=yes";

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn large_integer_keys() {
        let text = "species_db={\n\t956301313={\n\t\tname_list=\"MACHINE4\"\n\t}\n\t16777220={\n\t\tname_list=\"LITHOID4\"\n\t}\n}";

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn newline_between_equals_and_brace_dict() {
        let text = "flag=\n{\n\ticon=\n\t{\n\t\tcategory=\"infernal\"\n\t\tfile=\"infernal_11.dds\"\n\t}\n}";

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn empty_brace_with_newline() {
        let text = "traits=\n{\n}";

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn set_of_collections_with_newline() {
        let text = "player=\n{\n\t\n\t{\n\t\tname=\"SemanticallyInvalid\"\n\t\tcountry=0\n\t}\n \n\t{\n\t\tname=\"Mountny\"\n\t\tcountry=1\n\t}\n \n}";

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn trailing_whitespace() {
        let text = "key=val\n";

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn multiline_string_value() {
        let text = "effect=\"Astral Threads Found:\nResources: Yastral_threads|1 50.00!\"";

        let result = root(text);
        assert_result_ok(result);
    }
}
