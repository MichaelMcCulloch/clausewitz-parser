use std::time::Duration;

use nom::{combinator::map, FindSubstring};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use super::{bracketed::hash_map, val::Val, Res};

pub fn root<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(hash_map, Val::Dict)(input)
}

pub fn cheat_root<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    let mut indices = vec![];
    let mut after = input;
    while let Some(index) = after.find_substring("\n}\n") {
        let split = after.split_at(index + 3);

        indices.push(split.0);
        after = split.1;
    }
    indices.push(after);

    let res = Val::Dict(
        indices
            .par_iter()
            .filter_map(|string| {
                if string.starts_with("version=")
                    || string.starts_with("player=")
                    || string.starts_with("country=")
                {
                    match root(string) {
                        Ok((_, Val::Dict(dict))) => Some(dict),
                        Ok(_) => None,
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
            .flat_map(|v| v)
            .collect(),
    );
    let res = Ok(("", res));

    res
}

#[cfg(test)]
mod tests {
    use crate::{clausewitz::tests::helper::assert_result_ok, key_value};
    #[test]
    fn root__key_identifier_pairs__ok() {
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

        let result = cheat_root(&text);

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

        let result = root(&text);
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
    fn quoted__key__ok() {
        let text = r###""The name Of A Ship"=0"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn empty__set__set() {
        let text = r###"empty_set={}"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn root__set_of_strings__accepted() {
        let text = r###"set_of_strings={
                "Ancient Relics Story Pack"
                "Anniversary Portraits"
                "Apocalypse"
            }"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn array__of__arrays() {
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
    fn identifier__with__underscore() {
        let text = r###"identifier=identi_fire"###;

        let result = root(text);
        assert_result_ok(result);
    }

    #[test]
    fn dict__key_identifier_pairs__ok() {
        let text = r###"dict={
                alpha=a
                beta=b
                cthulhu=ilhjok
            }"###;

        let result = root(text);
        assert_result_ok(result);
    }
}
