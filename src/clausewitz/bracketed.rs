use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::{char, digit1},
    combinator::{cut, map, map_res, recognize, verify},
    multi::separated_list0,
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};

use super::{
    quoted::string_literal_contents,
    simd::{take_simd_identifier, take_simd_not_token},
    space::{opt_space, req_space},
    val::Val,
    value::value,
    Res,
};

#[inline(always)]
pub fn unquoted_key(input: &str) -> Res<&str, &str> {
    verify(take_simd_identifier, |s: &str| {
        !s.is_empty() //&& !(is_digit(s.chars().next().unwrap()))
    })(input)
}

#[inline(always)]
pub fn quoted_key(input: &str) -> Res<&str, &str> {
    delimited(char('\"'), string_literal_contents, char('\"'))(input)
}

#[inline(always)]
pub fn key(input: &str) -> Res<&str, &str> {
    alt((unquoted_key, quoted_key))(input)
}

#[inline(always)]
pub fn key_value<'a>(input: &'a str) -> Res<&'a str, (&'a str, Val<'a>)> {
    separated_pair(
        preceded(opt_space, key),
        cut(preceded(opt_space, char('='))),
        preceded(opt_space, value),
    )(input)
}

#[inline(always)]
pub fn hash_map<'a>(input: &'a str) -> Res<&'a str, Vec<(&'a str, Val<'a>)>> {
    separated_list0(req_space, key_value)(input)
}

#[inline(always)]
pub fn dict<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(hash_map, Val::Dict)(input)
}

#[inline(always)]
pub fn number_value<'a>(input: &'a str) -> Res<&'a str, (u64, Val<'a>)> {
    separated_pair(
        preceded(
            opt_space,
            map_res(
                verify(recognize(digit1), |s: &str| !s.is_empty()),
                str::parse,
            ),
        ),
        cut(preceded(opt_space, char('='))),
        preceded(opt_space, value),
    )(input)
}

#[inline(always)]
pub fn array<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(
        separated_list0(req_space, number_value),
        |mut number_value_pairs| {
            number_value_pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
            Val::Array(number_value_pairs)
        },
    )(input)
}

#[inline(always)]
pub fn set<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    alt((
        map(separated_list0(req_space, value), |s: Vec<Val>| Val::Set(s)),
        map(opt_space, |_s: &str| Val::Set(vec![])),
    ))(input)
}

#[inline(always)]
pub fn set_of_collections<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(separated_list0(req_space, bracketed), Val::Set)(input)
}

#[inline(always)]
pub fn contents<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    let (_remainder, (maybe_key_number_identifier, next_token)) =
        pair(take_simd_not_token, take(1_usize))(input)?;

    match next_token {
        "}" => cut(set)(input),
        _ => {
            match (
                next_token,
                take_simd_identifier(maybe_key_number_identifier)
                    .map(|s| s.1.parse::<i64>().is_ok())
                    .unwrap_or(false),
            ) {
                ("=", true) => cut(array)(input),
                ("=", false) => cut(dict)(input),
                ("{", true) => cut(numbered_dict)(input),
                ("{", false) => cut(set_of_collections)(input),
                (_, _) => {
                    panic!()
                }
            }
        }
    }
}

#[inline(always)]
pub fn bracketed<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    delimited(
        char('{'),
        cut(delimited(opt_space, contents, opt_space)),
        char('}'),
    )(input)
}

#[inline(always)]
pub fn numbered_dict<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(
        tuple((
            map_res(
                verify(recognize(digit1), |s: &str| !s.is_empty()),
                str::parse,
            ),
            req_space,
            delimited(
                char('{'),
                delimited(opt_space, hash_map, opt_space),
                char('}'),
            ),
        )),
        |(number, _, map): (i64, &str, Vec<(&'a str, Val<'a>)>)| Val::NumberedDict(number, map),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clausewitz::tests::helper::assert_result_ok;
    #[test]
    fn bracketed_dict_dict() {
        let text = r###"{
			first="first"
			second="second"
	}"###;
        let result = bracketed(text);
        assert_result_ok(result)
    }

    #[test]
    fn bracketed_array_array() {
        let text = r###"{
		0="first"
		1="second"
	}"###;
        let result = bracketed(text);
        assert_result_ok(result)
    }

    #[test]
    fn bracketed_set_set() {
        let text = r###"{
		"first"
		"second"
	}"###;
        let result = bracketed(text);
        assert_result_ok(result)
    }
    #[cfg(test)]
    mod key_value {
        use crate::clausewitz::{bracketed::key_value, tests::helper::assert_result_ok};

        #[test]
        fn key_value_unquoted_accepted() {
            let text = r###"key.0="value""###;
            let result = key_value(text);
            assert_result_ok(result)
        }

        #[test]
        fn key_value_quoted_accepted() {
            let text = r###""key.0"=0"###;
            let result = key_value(text);
            assert_result_ok(result)
        }
        #[test]
        fn key_value_begins_with_number_quoted_accepted() {
            let text = r###""0_key.0"=0"###;
            let result = key_value(text);
            assert_result_ok(result)
        }
        #[test]
        fn key_value_begins_with_number_unquoted_accepted() {
            let text = r###"0_key.0=0"###;
            let result = key_value(text);
            assert_result_ok(result)
        }
    }
    #[cfg(test)]
    mod dict {}

    #[cfg(test)]
    mod number_value {}

    #[cfg(test)]
    mod array {}
    #[cfg(test)]
    mod set {}
}
