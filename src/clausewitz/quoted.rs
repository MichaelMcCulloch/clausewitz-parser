use super::{
    simd::{take_while_simd, STRING_LITTERAL_CONTENT_RANGES},
    tables::is_string_litteral_contents,
    Res, Val,
};
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
use time::{Date, Month};

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
        |date: Date| Val::Date(date),
    )(input)
}

pub fn map_to_date<'a>(s: &'a str) -> anyhow::Result<Date> {
    let parts: Vec<&'a str> = s.split(".").collect();

    let year: i32 = parts
        .get(0)
        .ok_or(DateParseError {
            err: String::from("Too Short"),
        })?
        .parse()?;
    let month = match *parts.get(1).ok_or(DateParseError {
        err: String::from("Too Short"),
    })? {
        "01" => Ok(Month::January),
        "02" => Ok(Month::February),
        "03" => Ok(Month::March),
        "04" => Ok(Month::April),
        "05" => Ok(Month::May),
        "06" => Ok(Month::June),
        "07" => Ok(Month::July),
        "08" => Ok(Month::August),
        "09" => Ok(Month::September),
        "10" => Ok(Month::October),
        "11" => Ok(Month::November),
        "12" => Ok(Month::December),
        _ => Err(DateParseError {
            err: String::from("Months beyond December are not supported, dummy!"),
        }),
    };
    let day: u8 = parts
        .get(2)
        .ok_or(DateParseError {
            err: String::from("Too Short"),
        })?
        .parse()?;
    Ok(Date::from_calendar_date(year, month?, day)?)
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
    use time::{Date, Month};

    #[test]
    fn quoted__date__date() {
        let text = "\"2200.01.01\"";
        let (_remainder, parse_output) = quoted(text).unwrap();
        assert_eq!(
            parse_output,
            Val::Date(Date::from_calendar_date(2200, Month::January, 01).unwrap())
        );
    }

    #[test]
    fn quoted__not_date__string() {
        let text = "\"2200.011\"";
        let (_remainder, parse_output) = quoted(text).unwrap();
        assert_eq!(parse_output, Val::StringLiteral("2200.011"));
    }

    #[cfg(test)]
    mod date_test {
        use time::Month;

        use super::*;
        #[test]
        fn date__decimal_separated_yyyy_mm_date__accepted() {
            let text = "2200.01.01";
            let (_remainder, parse_output) = date(text).unwrap();
            assert_eq!(
                parse_output,
                Val::Date(Date::from_calendar_date(2200, Month::January, 01).unwrap())
            );
        }

        #[test]
        fn date__4digit_year__accepted() {
            let text = "2200.01.01";
            let (_remainder, parse_output) = date(text).unwrap();
            assert_eq!(
                parse_output,
                Val::Date(Date::from_calendar_date(2200, Month::January, 01).unwrap())
            );
        }

        #[test]
        fn date__3digit_year__accepted() {
            let text = "200.01.01";
            let (_remainder, parse_output) = date(text).unwrap();
            assert_eq!(
                parse_output,
                Val::Date(Date::from_calendar_date(200, Month::January, 01).unwrap())
            );
        }

        #[test]
        fn date__2digit_year__accepted() {
            let text = "20.01.01";
            let (_remainder, parse_output) = date(text).unwrap();
            assert_eq!(
                parse_output,
                Val::Date(Date::from_calendar_date(20, Month::January, 01).unwrap())
            );
        }

        #[test]
        fn date__1digit_year__accepted() {
            let text = "2.01.01";
            let (_remainder, parse_output) = date(text).unwrap();
            assert_eq!(
                parse_output,
                Val::Date(Date::from_calendar_date(2, Month::January, 01).unwrap())
            );
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
        fn string_literal__decimal_separated_yyyy_mm_string_litteral__accepted() {
            let (remainder_quote, result_quote) = string_literal("\"").unwrap();
            let (remainder_eq, result_eq) = string_literal("=").unwrap();
            let (remainder_lbracket, result_lbracket) = string_literal("{").unwrap();
            let (remainder_rbracket, result_rbracket) = string_literal("}").unwrap();

            assert_eq!(result_quote, Val::StringLiteral(""));
            assert_eq!(result_eq, Val::StringLiteral(""));
            assert_eq!(result_lbracket, Val::StringLiteral(""));
            assert_eq!(result_rbracket, Val::StringLiteral(""));

            assert_eq!(remainder_quote, "\"");
            assert_eq!(remainder_eq, "=");
            assert_eq!(remainder_lbracket, "{");
            assert_eq!(remainder_rbracket, "}");
        }
    }
}
