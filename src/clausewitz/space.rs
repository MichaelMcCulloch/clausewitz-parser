use super::{simd::take_simd_space, Res};
use nom::combinator::verify;
#[inline(always)]
pub fn opt_space<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    take_simd_space(input)
}
#[inline(always)]
pub fn req_space<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    verify(opt_space, |spaces: &str| !spaces.is_empty())(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opt_space__empty_string__accepted() {
        let text = "";

        let (remainder, parse_output) = opt_space(text).unwrap();
        assert_eq!(remainder, "");
        assert_eq!(parse_output, "");
    }

    #[test]
    fn opt_space__all_space_chars__accepted() {
        let text = " \t\n\r";

        let (remainder, parse_output) = opt_space(text).unwrap();
        assert_eq!(remainder, "");
        assert_eq!(parse_output, " \t\n\r");
    }

    #[test]
    fn req_space__empty_string__rejected() {
        let text = "";
        assert!(req_space(text).is_err())
    }

    #[test]
    fn req_space__all_space_chars__accepted() {
        let text = " \t\n\r";

        let (remainder, parse_output) = req_space(text).unwrap();
        assert_eq!(remainder, "");
        assert_eq!(parse_output, " \t\n\r");
    }
}
