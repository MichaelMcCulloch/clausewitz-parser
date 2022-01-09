use nom::{
    character::complete::{char, digit1},
    combinator::{map, map_res, opt, recognize, verify},
    error::VerboseError,
    sequence::tuple,
};

use super::{
    simd::{take_while_simd, IDENTIFIER_RANGES},
    tables::{is_digit, is_identifier_char},
    Res, Val,
};
pub fn decimal<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(
        map_res(
            recognize(tuple((opt(char('-')), digit1, char('.'), digit1))),
            str::parse,
        ),
        |float: f64| Val::Decimal(float),
    )(input)
}

pub fn int<'a>(input: &'a str) -> Res<&'a str, i64> {
    map_res(
        verify(recognize(tuple((opt(char('-')), digit1))), |s: &str| {
            !s.is_empty()
        }),
        str::parse,
    )(input)
}

pub fn integer<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(int, |integer: i64| Val::Integer(integer))(input)
}

pub fn identifier<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    map(
        verify(
            take_while_simd::<'a, _, VerboseError<&'a str>>(is_identifier_char, IDENTIFIER_RANGES),
            |s: &str| !s.is_empty() && !(is_digit(s.chars().next().unwrap())),
        ),
        |s: &str| Val::Identifier(s),
    )(input)
}
use nom::branch::alt;

pub fn unquoted<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    alt((decimal, integer, identifier))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unquoted__integer__integer() {
        let text = "0";
        let (_remainder, parse_output) = unquoted(text).unwrap();
        assert_eq!(parse_output, Val::Integer(0));
    }
    #[test]
    fn unquoted__decimal__decimal() {
        let text = "0.0";
        let (_remainder, parse_output) = unquoted(text).unwrap();
        assert_eq!(parse_output, Val::Decimal(0.0));
    }
    #[test]
    fn unquoted__identifier__identifier() {
        let text = "zer0";
        let (_remainder, parse_output) = unquoted(text).unwrap();
        assert_eq!(parse_output, Val::Identifier("zer0"));
    }

    #[cfg(test)]
    mod identifier_tests {

        use super::*;

        #[test]
        fn identifire__alphanumeric_with_underscore_and_colon__accepted() {
            let text = "alpha_:numeric1234567890";
            let (remainder, parse_output) = identifier(text).unwrap();
            assert_eq!(parse_output, Val::Identifier(text));
            assert!(remainder.is_empty());
        }

        #[test]
        fn identifire__begins_with_number__rejectec() {
            let text = "0alpha_numeric1234567890";
            assert!(identifier(text).is_err());
        }

        #[test]
        fn identifire__empty__rejectec() {
            let text = "";
            assert!(identifier(text).is_err());
        }
    }
    #[cfg(test)]
    mod integer {

        use super::*;

        #[test]
        fn integer__empty__rejected() {
            let text = "";
            assert!(integer(text).is_err());
        }
        #[test]
        fn integer__zero__accepted() {
            let text = "0";
            let (remainder, parse_output) = integer(text).unwrap();
            assert_eq!(parse_output, Val::Integer(0));
            assert!(remainder.is_empty());
        }

        #[test]
        fn integer__negative_number__accepted() {
            let text = "-1";
            let (remainder, parse_output) = integer(text).unwrap();
            assert_eq!(parse_output, Val::Integer(-1));
            assert!(remainder.is_empty());
        }

        #[test]
        fn integer__all_digits__accepted() {
            let text = "1234567890";
            let (remainder, parse_output) = integer(text).unwrap();
            assert_eq!(parse_output, Val::Integer(1234567890));
            assert!(remainder.is_empty());
        }

        #[test]
        fn integer__dots__accepted_up_to_dot_then_remainder() {
            let text = "-12345.6789";
            let (remainder, parse_output) = integer(text).unwrap();
            assert_eq!(parse_output, Val::Integer(-12345));
            assert_eq!(remainder, ".6789");
        }

        #[test]
        fn integer__letters__int_up_to_letter_then_remainder() {
            let text = "-1234567d89.098098";
            let (remainder, parse_output) = integer(text).unwrap();
            assert_eq!(parse_output, Val::Integer(-1234567));
            assert_eq!(remainder, "d89.098098");
        }
    }

    #[cfg(test)]
    mod decimal_tests {

        use super::*;

        #[test]
        fn decimal__small_number__accepted() {
            let text = "0.00001011110110132";
            let (remainder, parse_output) = decimal(text).unwrap();
            assert_eq!(parse_output, Val::Decimal(0.00001011110110132));
            assert!(remainder.is_empty());
        }

        #[test]
        fn decimal__negative_number__accepted() {
            let text = "-0.1";
            let (remainder, parse_output) = decimal(text).unwrap();
            assert_eq!(parse_output, Val::Decimal(-0.1));
            assert!(remainder.is_empty());
        }

        #[test]
        fn decimal__all_digits__accepted() {
            let text = "-12345.6789";
            let (remainder, parse_output) = decimal(text).unwrap();
            assert_eq!(parse_output, Val::Decimal(-12345.6789));
            assert!(remainder.is_empty());
        }

        #[test]
        fn decimal__too_many_dots__accepted_with_remainder() {
            let text = "-12345.6789.098098";
            let (remainder, parse_output) = decimal(text).unwrap();
            assert_eq!(parse_output, Val::Decimal(-12345.6789));
            assert_eq!(remainder, ".098098");
        }

        #[test]
        fn decimal__letters__float_up_to_letter_then_remainder() {
            let text = "-12345.67d89.098098";
            let (remainder, parse_output) = decimal(text).unwrap();
            assert_eq!(parse_output, Val::Decimal(-12345.67));
            assert_eq!(remainder, "d89.098098");
        }
    }
}
