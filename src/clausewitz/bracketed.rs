use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::{char, digit1},
    combinator::{cut, map, map_res, recognize, verify},
    error::ParseError,
    multi::separated_list0,
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    IResult, Parser,
};

use super::{
    quoted::string_literal_contents,
    simd::{take_simd_identifier, take_simd_not_token},
    space::{opt_space, req_space},
    unquoted::integer,
    val::Val,
    value::value,
    Res,
};

#[inline(always)]
pub fn unquoted_key<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    verify(take_simd_identifier, |s: &str| {
        !s.is_empty() //&& !(is_digit(s.chars().next().unwrap()))
    })(input)
}

#[inline(always)]
pub fn quoted_key<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    delimited(char('\"'), string_literal_contents, char('\"'))(input)
}

#[inline(always)]
pub fn key<'a>(input: &'a str) -> Res<&'a str, &'a str> {
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
pub fn number_value<'a>(input: &'a str) -> Res<&'a str, (usize, Val<'a>)> {
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
        |number_value_pairs| Val::Array(fold_into_array(number_value_pairs)),
    )(input)
}

#[inline(always)]
pub fn fold_into_array<'a>(mut tuple_vec: Vec<(usize, Val<'a>)>) -> Vec<Val<'a>> {
    tuple_vec.sort_by(|(a_index, _), (b_index, _)| a_index.partial_cmp(b_index).unwrap());
    tuple_vec.into_iter().map(|(_, val)| val).collect()
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
    map(separated_list0(req_space, bracketed), |vals| Val::Set(vals))(input)
}

#[inline(always)]
pub fn triple<I, O1, O2, O3, E: ParseError<I>, F, G, H>(
    mut first: F,
    mut second: G,
    mut third: H,
) -> impl FnMut(I) -> IResult<I, (O1, O2, O3), E>
where
    F: Parser<I, O1, E>,
    G: Parser<I, O2, E>,
    H: Parser<I, O3, E>,
{
    move |input: I| {
        let (input, o1) = first.parse(input)?;
        let (input, o2) = second.parse(input)?;
        third.parse(input).map(|(i, o3)| (i, (o1, o2, o3)))
    }
}
#[inline(always)]
pub fn contents<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    let (_remainder, (maybe_key_number_identifier, next_token)) =
        pair(take_simd_not_token, take(1 as usize))(input)?;

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
    fn bracketed__dict__dict() {
        let text = r###"{
			first="first"
			second="second"
	}"###;
        let result = bracketed(text);
        assert_result_ok(result)
    }

    #[test]
    fn bracketed__array__array() {
        let text = r###"{
		0="first"
		1="second"
	}"###;
        let result = bracketed(text);
        assert_result_ok(result)
    }

    #[test]
    fn bracketed__set__set() {
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
        fn key_value__unquoted__accepted() {
            let text = r###"key.0="value""###;
            let result = key_value(text);
            assert_result_ok(result)
        }

        #[test]
        fn key_value__quoted__accepted() {
            let text = r###""key.0"=0"###;
            let result = key_value(text);
            assert_result_ok(result)
        }
        #[test]
        fn key_value__begins_with_number_quoted__accepted() {
            let text = r###""0_key.0"=0"###;
            let result = key_value(text);
            assert_result_ok(result)
        }
        #[test]
        fn key_value__begins_with_number_unquoted__accepted() {
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
