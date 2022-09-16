use std::{
    arch::x86_64::{
        _mm_cmpestri, _mm_loadu_si128, _SIDD_CMP_RANGES, _SIDD_LEAST_SIGNIFICANT, _SIDD_UBYTE_OPS,
    },
    cmp::min,
    f32::consts::E,
    ops::RangeFrom,
    path::Iter,
    process::exit,
    str::{CharIndices, Chars},
    vec,
};

use nom::{
    branch::alt,
    bytes::complete::{take, take_while},
    character::complete::{char, digit1},
    combinator::{cut, map, map_res, opt, recognize, verify},
    error::{ParseError, VerboseError, VerboseErrorKind},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, tuple},
    AsChar, IResult, InputIter, InputLength, Needed, Parser, Slice,
};
const CHUNK_SIZE: usize = 16;
use super::tables::{
    identifier_table, is_digit, is_space, is_string_litteral_contents, space_table,
    string_literal_content_table, token_table,
};
mod isp;
use isp::*;
type SkimResult<X, PARSED> = IResult<X, PARSED, VerboseError<X>>;

pub fn take_simd_spaces<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, &'a str> {
    todo!()
}
pub fn opt_space<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    take_while(|character| space_table()[character as usize])(input)
}
pub fn req_space<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    verify(opt_space, |spaces: &InputSearchPair| {
        !spaces.slice.is_empty()
    })(input)
}
pub fn key<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    alt((unquoted_key, quoted_key))(input)
}
pub fn quoted_key<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    delimited(char('\"'), string_literal_contents, char('\"'))(input)
}

pub fn string_literal_contents<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    take_while(|char| string_literal_content_table()[char as usize])(input)
}
pub fn unquoted_key<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    let first = take_while(|char| identifier_table()[char as usize]);
    verify(first, |key: &InputSearchPair| !key.slice.is_empty())(input)
}
pub fn date<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    recognize(tuple((digit1, char('.'), digit1, char('.'), digit1)))(input)
}
pub fn string_literal<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    take_while(|char| string_literal_content_table()[char as usize])(input)
}

///I think if we reach quoted or unquoted, we've found our value
pub fn quoted<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    map(
        delimited(char('\"'), cut(alt((date, string_literal))), char('\"')),
        |isp| vec![isp],
    )(input)
}
pub fn integer<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    verify(recognize(tuple((opt(char('-')), digit1))), |s: &str| {
        !s.is_empty()
    })(input)
}
pub fn decimal<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    recognize(tuple((opt(char('-')), digit1, char('.'), digit1)))(input)
}
pub fn identifier<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, InputSearchPair<'a, 'b>> {
    verify(
        take_while(|char| identifier_table()[char as usize]),
        |s: &str| !s.is_empty() && !(is_digit(s.chars().next().unwrap())),
    )(input)
}
///I think if we reach quoted or unquoted, we've found our value
pub fn unquoted<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    map(alt((date, decimal, integer, identifier)), |isp| vec![isp])(input)
}
pub fn value<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    alt((bracketed, quoted, unquoted))(input)
}

pub fn bracketed<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    delimited(
        char('{'),
        cut(delimited(opt_space, contents, opt_space)),
        char('}'),
    )(input)
}

pub fn set<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    alt((
        map(separated_list0(req_space, value), |vvec| {
            vvec.into_iter().flat_map(|vec| vec).collect()
        }),
        map(opt_space, |_s| vec![]),
    ))(input)
}

pub fn number_value<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    let (mut rem_number, number) = preceded(
        opt_space,
        verify(recognize(digit1), |isp: &InputSearchPair| {
            !isp.slice.is_empty()
        }),
    )(input)?;

    if number.slice == number.search_path[number.search_path_index] {
        rem_number = InputSearchPair {
            slice: rem_number.slice,
            search_path: rem_number.search_path,
            search_path_index: rem_number.search_path_index + 1,
        };
        let (rem_eq, _) = cut(preceded(opt_space, char('=')))(rem_number)?;
        let (mut rem_val, val) = preceded(opt_space, value)(rem_eq)?;

        //since we may come back to this in another iteration of the separated list that called it, we need to re increment the key for it's next loop
        rem_val = InputSearchPair {
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
}

pub fn array<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
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

pub fn dict<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    (search_hashmap)(input)
}

pub fn set_of_collections<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    map(separated_list0(req_space, bracketed), |vals| {
        vals.into_iter().flat_map(|f| f).collect()
    })(input)
}
pub fn numbered_dict<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
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

pub fn contents<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    let (remainder, maybe_key_number_identifier) =
        take_while(move |character| !token_table()[character as usize])(input)?;

    let (_remainder, next_token) = take(1 as usize)(remainder)?;

    if next_token.slice == "}" {
        return cut(set)(input);
    } else if next_token.slice == "=" {
        let (_rem, maybe_ident) =
            take_while(|char| identifier_table()[char as usize])(maybe_key_number_identifier)?;
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
fn key_value<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    match preceded(opt_space, key)(input) {
        Ok((mut rem_key, key)) => {
            if key.slice == key.search_path[key.search_path_index] {
                // found the key, search the value for the NEXT element in the key
                rem_key = InputSearchPair {
                    slice: rem_key.slice,
                    search_path: rem_key.search_path,
                    search_path_index: rem_key.search_path_index + 1,
                };
                let (rem_eq, _) = cut(preceded(opt_space, char('=')))(rem_key)?;
                let (mut rem_val, val) = preceded(opt_space, value)(rem_eq)?;

                //since we may come back to this in another iteration of the separated list that called it, we need to re increment the key for it's next loop
                rem_val = InputSearchPair {
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
        }
        Err(e) => Err(e),
    }
}

/// This should return a list the values at found paths
fn search_hashmap<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    separated_list0(req_space, key_value)(input)
        .map(|(isp, vec)| (isp, vec.into_iter().flat_map(|opt| opt).collect::<Vec<_>>()))
}

pub fn search_document<'a, 'b>(
    input: InputSearchPair<'a, 'b>,
) -> SkimResult<InputSearchPair<'a, 'b>, Vec<InputSearchPair<'a, 'b>>> {
    search_hashmap(input)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::exit;
    use std::sync::Arc;

    use super::*;
    #[test]
    fn search_document_test() {
        let text = r###"version="Cepheus v3.4.5"
        version_control_revision=95132
        name="mp_Custodianship"
        date="2230.12.01"
        required_dlcs={
            "Ancient Relics Story Pack"
            "Anniversary Portraits"
            "Apocalypse"
            "Distant Stars Story Pack"
            "Federations"
            "Horizon Signal"
            "Humanoids Species Pack"
            "Leviathans Story Pack"
            "Lithoids Species Pack"
            "Megacorp"
            "Necroids Species Pack"
            "Nemesis"
            "Overlord"
            "Plantoids Species Pack"
            "Synthetic Dawn Story Pack"
            "Utopia"
        }
        player_portrait="sd_hum_robot"
        flag={
            icon={
                category="human"
                file="flag_human_8.dds"
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
        }
        meta_fleets=30
        meta_planets=8
        "###;

        let input = InputSearchPair::create(text, "flag.icon.category");
        // let input = InputSearchPair::create(text, "flag.icon");//fails

        let (rem, opt) = search_document(input).unwrap();
        println!("{:?}", opt);
        assert!(!opt.is_empty());
        let expected = opt.first().unwrap();
        assert_eq!(&"Custodianship", &expected.slice);
    }
    use super::*;
}
