use crate::clausewitz_value::Val;

use super::{
    simd::{take_while_simd, STRING_LITTERAL_CONTENT_RANGES},
    tables::is_string_litteral_contents,
    Res,
};
use chrono::NaiveDate;
use nom::{
    branch::alt,
    character::complete::{char, digit1},
    combinator::{cut, map, map_res, recognize},
    error::VerboseError,
    sequence::{delimited, tuple},
};
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

#[derive(Debug, PartialEq)]
pub struct DateParseError {
    err: String,
}

impl Error for DateParseError {}

impl Display for DateParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}

pub fn date<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(
        map_res(
            recognize(tuple((digit1, char('.'), digit1, char('.'), digit1))),
            map_to_date,
        ),
        |date: NaiveDate| Val::Date(date),
    )(input)
}

pub fn map_to_date<'a>(s: &'a str) -> anyhow::Result<NaiveDate> {
    let parts: Vec<&'a str> = s.split(".").collect();

    let year = parts
        .get(0)
        .ok_or(DateParseError {
            err: String::from("Too Short"),
        })?
        .parse()?;
    let month = parts
        .get(1)
        .ok_or(DateParseError {
            err: String::from("Too Short"),
        })?
        .parse()?;
    let day = parts
        .get(2)
        .ok_or(DateParseError {
            err: String::from("Too Short"),
        })?
        .parse()?;

    //TODO: if the date really matters, find a way to allow leap years, ie feb 29th
    Ok(match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => date,
        None => NaiveDate::from_ymd(0, 1, 1),
    })
}

pub fn string_literal_contents<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    take_while_simd::<'a, _, VerboseError<&'a str>>(
        is_string_litteral_contents,
        STRING_LITTERAL_CONTENT_RANGES,
    )(input)
}

pub fn string_literal<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(string_literal_contents, |s: &str| Val::StringLiteral(s))(input)
}

pub fn quoted<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    delimited(char('\"'), cut(alt((date, string_literal))), char('\"'))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted__date__date() {
        let text = "\"2200.01.01\"";
        let (_remainder, parse_output) = quoted(text).unwrap();
        assert_eq!(parse_output, Val::Date(NaiveDate::from_ymd(2200, 1, 1)));
    }

    #[test]
    fn quoted__not_date__string() {
        let text = "\"2200.011\"";
        let (_remainder, parse_output) = quoted(text).unwrap();
        assert_eq!(parse_output, Val::StringLiteral("2200.011"));
    }

    #[cfg(test)]
    mod date_test {

        use super::*;
        #[test]
        fn date__decimal_separated_yyyy_mm_date__accepted() {
            let text = "2200.01.01";
            let (_remainder, parse_output) = date(text).unwrap();
            assert_eq!(parse_output, Val::Date(NaiveDate::from_ymd(2200, 1, 01)));
        }

        #[test]
        fn date__4digit_year__accepted() {
            let text = "2200.01.01";
            let (_remainder, parse_output) = date(text).unwrap();
            assert_eq!(parse_output, Val::Date(NaiveDate::from_ymd(2200, 1, 01)));
        }

        #[test]
        fn date__3digit_year__accepted() {
            let text = "200.01.01";
            let (_remainder, parse_output) = date(text).unwrap();
            assert_eq!(parse_output, Val::Date(NaiveDate::from_ymd(200, 1, 01)));
        }

        #[test]
        fn date__2digit_year__accepted() {
            let text = "20.01.01";
            let (_remainder, parse_output) = date(text).unwrap();
            assert_eq!(parse_output, Val::Date(NaiveDate::from_ymd(20, 1, 01)));
        }

        #[test]
        fn date__1digit_year__accepted() {
            let text = "2.01.01";
            let (_remainder, parse_output) = date(text).unwrap();
            assert_eq!(parse_output, Val::Date(NaiveDate::from_ymd(2, 1, 01)));
        }
    }

    #[cfg(test)]
    mod string_literal_test {

        use super::*;
        #[test]
        fn string_literal__string__accepted() {
            let text = "this is a string with a bun1234567890ch of special characters!@#$%^&*(_()";
            let (_remainder, parse_output) = string_literal(text).unwrap();
            assert_eq!(parse_output, Val::StringLiteral(text));
        }
        #[test]
        fn string_literal__accent__accepted() {
            let text = "Rivén's Burrow";
            let (_remainder, parse_output) = string_literal(text).unwrap();
            assert_eq!(parse_output, Val::StringLiteral(text));
        }

        #[test]
        fn string_literal__decimal_separated_yyyy_mm_string_litteral__accepted() {
            let (remainder_quote, result_quote) = string_literal("\"").unwrap();

            assert_eq!(result_quote, Val::StringLiteral(""));

            assert_eq!(remainder_quote, "\"");
        }
    }
}
