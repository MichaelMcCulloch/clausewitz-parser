use std::vec;

use nom::{
    branch::alt,
    bytes::complete::{take, take_while},
    character::complete::{char, digit1},
    combinator::{cut, map, opt, recognize, verify},
    error::{ParseError, VerboseError, VerboseErrorKind},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, tuple},
    AsChar, IResult, InputIter, InputLength, Needed, Parser, Slice,
};
const CHUNK_SIZE: usize = 16;
use super::{
    simd::{take_simd_identifier, take_simd_not_token, take_simd_space, take_simd_string_literal},
    tables::{identifier_table, is_digit},
};
pub mod isp;
use isp::*;
type SR<X, PARSED> = IResult<X, PARSED, VerboseError<X>>;

pub fn opt_space<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    match take_simd_space(input.slice) {
        Ok((rem, spaces)) => Ok((
            ISP {
                slice: rem,
                search_path: input.search_path,
                search_path_index: input.search_path_index,
            },
            ISP {
                slice: spaces,
                search_path: input.search_path,
                search_path_index: input.search_path_index,
            },
        )),
        Err(e) => Err(e.map(|e| VerboseError {
            errors: e
                .errors
                .into_iter()
                .map(|(str, vbe)| (input, VerboseErrorKind::Context("whatever")))
                .collect(),
        })),
    }
}
pub fn req_space<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    verify(opt_space, |spaces: &ISP| !spaces.slice.is_empty())(input)
}
pub fn key<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    alt((unquoted_key, quoted_key))(input)
}
pub fn quoted_key<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    delimited(char('\"'), string_literal_contents, char('\"'))(input)
}

pub fn string_literal_contents<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    match take_simd_string_literal(input.slice) {
        Ok((rem, spaces)) => Ok((
            ISP {
                slice: rem,
                search_path: input.search_path,
                search_path_index: input.search_path_index,
            },
            ISP {
                slice: spaces,
                search_path: input.search_path,
                search_path_index: input.search_path_index,
            },
        )),
        Err(e) => Err(e.map(|e| VerboseError {
            errors: e
                .errors
                .into_iter()
                .map(|(str, vbe)| (input, VerboseErrorKind::Context("whatever")))
                .collect(),
        })),
    }
}
pub fn unquoted_key<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    verify(identifier_simd, |key: &ISP| !key.slice.is_empty())(input)
}
pub fn date<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    recognize(tuple((digit1, char('.'), digit1, char('.'), digit1)))(input)
}
pub fn string_literal<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    string_literal_contents(input)
}

///I think if we reach quoted or unquoted, we've found our value
pub fn quoted<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    map(
        delimited(char('\"'), cut(alt((date, string_literal))), char('\"')),
        |isp| vec![isp],
    )(input)
}
pub fn integer<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    verify(recognize(tuple((opt(char('-')), digit1))), |s: &str| {
        !s.is_empty()
    })(input)
}
pub fn decimal<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    recognize(tuple((opt(char('-')), digit1, char('.'), digit1)))(input)
}
pub fn identifier<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    verify(identifier_simd, |s: &str| {
        !s.is_empty() && !(is_digit(s.chars().next().unwrap()))
    })(input)
}
///I think if we reach quoted or unquoted, we've found our value
pub fn unquoted<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    map(alt((date, decimal, integer, identifier)), |isp| vec![isp])(input)
}
pub fn value<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    alt((bracketed, quoted, unquoted))(input)
}

pub fn bracketed<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    delimited(
        char('{'),
        cut(delimited(opt_space, contents, opt_space)),
        char('}'),
    )(input)
}
pub fn term_bracketed<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    delimited(
        char('{'),
        cut(delimited(opt_space, contents, opt_space)),
        char('}'),
    )(input)
}

pub fn set<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    alt((
        map(separated_list0(req_space, value), |vvec| {
            vvec.into_iter().flat_map(|vec| vec).collect()
        }),
        map(opt_space, |_s| vec![]),
    ))(input)
}

pub fn number_value<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    let (mut rem_number, number) = preceded(
        opt_space,
        verify(recognize(digit1), |isp: &ISP| !isp.slice.is_empty()),
    )(input)?;

    if number.search_path_index < 10 {
        if number.slice == number.search_path[number.search_path_index] {
            rem_number = ISP {
                slice: rem_number.slice,
                search_path: rem_number.search_path,
                search_path_index: rem_number.search_path_index + 1,
            };
            let (rem_eq, _) = cut(preceded(opt_space, char('=')))(rem_number)?;
            let (mut rem_val, val) = preceded(opt_space, value)(rem_eq)?;

            //since we may come back to this in another iteration of the separated list that called it, we need to re increment the key for it's next loop
            rem_val = ISP {
                slice: rem_val.slice,
                search_path: rem_val.search_path,
                search_path_index: rem_val.search_path_index - 1,
            };
            Ok((rem_val, val))
        } else {
            let (rem_eq, _) = cut(preceded(opt_space, char('=')))(rem_number)?;
            let (rem_val, _) = preceded(opt_space, value)(rem_eq)?;
            Ok((rem_val, vec![]))
        }
    } else {
        let (rem_eq, _) = cut(preceded(opt_space, char('=')))(rem_number)?;
        let (rem_val, val) = preceded(opt_space, value)(rem_eq)?;
        Ok((rem_val, val))
    }
}

pub fn array<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    let f = number_value(input);

    map(
        separated_list0(req_space, number_value),
        |number_value_pairs| {
            number_value_pairs
                .into_iter()
                .flat_map(|f| f)
                .collect::<Vec<_>>()
        },
    )(input)
}

pub fn dict<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    (search_hashmap)(input)
}

pub fn set_of_collections<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    map(separated_list0(req_space, bracketed), |vals| {
        vals.into_iter().flat_map(|f| f).collect()
    })(input)
}
pub fn numbered_dict<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    map(
        tuple((
            verify(recognize(digit1), |s: &str| !s.is_empty()),
            req_space,
            delimited(
                char('{'),
                delimited(opt_space, search_hashmap, opt_space),
                char('}'),
            ),
        )),
        |(number, _, map)| map,
    )(input)
}

pub fn contents<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    let (remainder, maybe_key_number_identifier) = not_token(input)?;

    let (_remainder, next_token) = take(1 as usize)(remainder)?;

    if next_token.slice == "}" {
        return cut(set)(input);
    } else if next_token.slice == "=" {
        let (_rem, maybe_ident) = identifier_simd(maybe_key_number_identifier)?;
        return if let Ok(_) = maybe_ident.slice.parse::<i64>() {
            cut(array)(input)
        } else {
            cut(dict)(input)
        };
    } else if next_token.slice == "{" {
        return if integer(maybe_key_number_identifier).is_ok() {
            cut(numbered_dict)(input)
        } else {
            cut(set_of_collections)(input)
        };
    } else {
        println!("AFTER: {}", input.slice);
        println!("{}", next_token.slice);
        panic!("Token = or }} not found, possibly missing a closing brace somewhere?")
    };
}

fn identifier_simd<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    match take_simd_identifier(input.slice) {
        Ok((rem, spaces)) => Ok((
            ISP {
                slice: rem,
                search_path: input.search_path,
                search_path_index: input.search_path_index,
            },
            ISP {
                slice: spaces,
                search_path: input.search_path,
                search_path_index: input.search_path_index,
            },
        )),
        Err(e) => Err(e.map(|e| VerboseError {
            errors: e
                .errors
                .into_iter()
                .map(|(str, vbe)| (input, VerboseErrorKind::Context("whatever")))
                .collect(),
        })),
    }
}

fn not_token<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, ISP<'a, 'b>> {
    match take_simd_not_token(input.slice) {
        Ok((rem, spaces)) => Ok((
            ISP {
                slice: rem,
                search_path: input.search_path,
                search_path_index: input.search_path_index,
            },
            ISP {
                slice: spaces,
                search_path: input.search_path,
                search_path_index: input.search_path_index,
            },
        )),
        Err(e) => Err(e.map(|e| VerboseError {
            errors: e
                .errors
                .into_iter()
                .map(|(str, vbe)| (input, VerboseErrorKind::Context("whatever")))
                .collect(),
        })),
    }
}
fn key_value<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    match preceded(opt_space, key)(input) {
        Ok((mut rem_key, key)) => {
            if key.search_path_index < 10 {
                if key.slice == key.search_path[key.search_path_index] {
                    // found the key, search the value for the NEXT element in the key
                    rem_key = ISP {
                        slice: rem_key.slice,
                        search_path: rem_key.search_path,
                        search_path_index: rem_key.search_path_index + 1,
                    };
                    let (rem_eq, _) = cut(preceded(opt_space, char('=')))(rem_key)?;
                    let (mut rem_val, val) = preceded(opt_space, value)(rem_eq)?;

                    //since we may come back to this in another iteration of the separated list that called it, we need to re increment the key for it's next loop
                    rem_val = ISP {
                        slice: rem_val.slice,
                        search_path: rem_val.search_path,
                        search_path_index: rem_val.search_path_index - 1,
                    };
                    Ok((rem_val, val))
                } else {
                    let (rem_eq, eq) = cut(preceded(opt_space, char('=')))(rem_key)?;
                    let (rem_val, val) = preceded(opt_space, value)(rem_eq)?;
                    Ok((rem_val, vec![]))
                }
            } else {
                let (rem_eq, eq) = cut(preceded(opt_space, char('=')))(rem_key)?;
                let (rem_val, val) = preceded(opt_space, value)(rem_eq)?;

                Ok((rem_val, val))
            }
        }
        Err(e) => Err(e),
    }
}

/// This should return a list the values at found paths
fn search_hashmap<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    separated_list0(req_space, key_value)(input)
        .map(|(isp, vec)| (isp, vec.into_iter().flat_map(|opt| opt).collect::<Vec<_>>()))
}

pub fn search_document<'a, 'b>(input: ISP<'a, 'b>) -> SR<ISP<'a, 'b>, Vec<ISP<'a, 'b>>> {
    search_hashmap(input)
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::process::exit;
    use std::sync::Arc;

    use memmap::Mmap;

    use super::*;
    #[test]
    fn search_document_test() {
        let filename =
            "/home/michael/Dev/Stellarust/clausewitz-parser/production_data/3.4.5.95132/2290.03.05/gamestate";
        let file = File::open(filename).expect("File not found");

        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {:?}", file)) };

        let str = String::from_utf8_lossy(&mmap[..]);
        let input = ISP::create(&str, "country.0.budget.current_month.income.country_base");
        // let input = InputSearchPair::create(text, "flag.icon");//fails

        let (rem, opt) = search_document(input).unwrap();
        println!("{:?}", opt);
        assert!(!opt.is_empty());
        let expected = opt.first().unwrap();
        assert_eq!(&"25.5", &expected.slice);
    }

    #[test]
    fn asdf() {
        let str = r###"country = {
            1 = one
            2 = two
        }"###;
        let input = ISP::create(&str, "country");
        // let input = InputSearchPair::create(text, "flag.icon");//fails

        let res = search_document(input);

        let (rem, opt) = res.unwrap();
        println!("{:?}", opt);
        assert!(!opt.is_empty());
        let expected = opt.first().unwrap();
        assert_eq!(&"25.5", &expected.slice);
    }
    use super::*;
}
