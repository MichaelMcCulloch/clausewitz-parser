use super::{simd::take_simd_space, Res};
use nom::combinator::verify;
#[inline(always)]
pub fn opt_space(input: &str) -> Res<&str, &str> {
    take_simd_space(input)
}
#[inline(always)]
pub fn req_space(input: &str) -> Res<&str, &str> {
    verify(opt_space, |spaces: &str| !spaces.is_empty())(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opt_space_empty_string_accepted() {
        let text = "";

        let (remainder, parse_output) = opt_space(text).unwrap();
        assert_eq!(remainder, "");
        assert_eq!(parse_output, "");
    }

    #[test]
    fn opt_space_all_space_chars_accepted() {
        let text = " \t\n\r";

        let (remainder, parse_output) = opt_space(text).unwrap();
        assert_eq!(remainder, "");
        assert_eq!(parse_output, " \t\n\r");
    }

    #[test]
    fn req_space_empty_string_rejected() {
        let text = "";
        assert!(req_space(text).is_err())
    }

    #[test]
    fn req_space_all_space_chars_accepted() {
        let text = " \t\n\r";

        let (remainder, parse_output) = req_space(text).unwrap();
        assert_eq!(remainder, "");
        assert_eq!(parse_output, " \t\n\r");
    }
}
