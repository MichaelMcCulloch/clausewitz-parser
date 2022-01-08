use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::{char, digit1},
    combinator::{cut, map, map_res, recognize, verify},
    error::VerboseError,
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, tuple},
};

use super::{
    quoted::string_literal_contents,
    simd::{take_while_simd, IDENTIFIER_RANGES, NOT_TOKEN_RANGES},
    space::{opt_space, req_space},
    tables::{is_digit, is_identifier_char, is_token},
    unquoted::integer,
    value::value,
    Res, Val,
};

pub fn unquoted_key<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    verify(
        take_while_simd::<'a, _, VerboseError<&'a str>>(is_identifier_char, IDENTIFIER_RANGES),
        |s: &str| !s.is_empty() && !(is_digit(s.chars().next().unwrap())),
    )(input)
}

pub fn quoted_key<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    delimited(char('\"'), string_literal_contents, char('\"'))(input)
}

pub fn key<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    alt((unquoted_key, quoted_key))(input)
}

pub fn key_value<'a>(input: &'a str) -> Res<&'a str, (&'a str, Val<'a>)> {
    separated_pair(
        preceded(opt_space, key),
        cut(preceded(opt_space, char('='))),
        preceded(opt_space, value),
    )(input)
}

pub fn hash_map<'a>(input: &'a str) -> Res<&'a str, Vec<(&'a str, Val<'a>)>> {
    separated_list0(req_space, key_value)(input)
}

pub fn dict<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(hash_map, Val::Dict)(input)
}

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

pub fn array<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(
        separated_list0(req_space, number_value),
        |number_value_pairs| Val::Array(fold_into_array(number_value_pairs)),
    )(input)
}

pub fn fold_into_array<'a>(mut tuple_vec: Vec<(usize, Val<'a>)>) -> Vec<Val<'a>> {
    tuple_vec.sort_by(|(a_index, _), (b_index, _)| a_index.partial_cmp(b_index).unwrap());
    tuple_vec.into_iter().map(|(_, val)| val).collect()
}

pub fn set<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    alt((
        map(separated_list0(req_space, value), |s: Vec<Val>| Val::Set(s)),
        map(opt_space, |_s: &str| Val::Set(vec![])),
    ))(input)
}

pub fn set_of_collections<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(separated_list0(req_space, bracketed), |vals| Val::Set(vals))(input)
}
pub fn contents<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    let (remainder, maybe_key_number_idenentifier): (&'a str, &'a str) =
        take_while_simd::<'a, _, VerboseError<&'a str>>(
            move |character| !is_token(character),
            NOT_TOKEN_RANGES,
        )(input)?;

    let (_remainder, next_token) = take(1 as usize)(remainder)?;

    if next_token == "}" {
        return cut(set)(input);
    } else if next_token == "=" {
        return if integer(maybe_key_number_idenentifier).is_ok() {
            cut(array)(input)
        } else {
            cut(dict)(input)
        };
    } else if next_token == "{" {
        return if integer(maybe_key_number_idenentifier).is_ok() {
            cut(numbered_dict)(input)
        } else {
            cut(set_of_collections)(input)
        };
    } else {
        println!("AFTER: {}", input);
        println!("{}", next_token);
        panic!("Token = or }} not found, possibly missing a closing brace somewhere?")
    };
}

pub fn bracketed<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    delimited(
        char('{'),
        cut(delimited(opt_space, contents, opt_space)),
        char('}'),
    )(input)
}

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
            let text = r###"key.0="value"
			"###;
            let result = key_value(text);
            assert_result_ok(result)
        }

        #[test]
        fn key_value__quoted__accepted() {
            let text = r###""key.0"=0
			"###;
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
