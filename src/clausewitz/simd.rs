use nom::bytes::complete::take_while;

use std::arch::x86_64::{
    _mm_cmpestri, _mm_loadu_si128, _SIDD_CMP_RANGES, _SIDD_LEAST_SIGNIFICANT, _SIDD_UBYTE_OPS,
};
use std::cmp::min;

//the range of all the characters which should be REJECTED
pub const SPACE_RANGES: &[u8; 16] = &[
    b'\x00', b'\x08', b'\x0e', b'\x1f', b'!', b'\xff', b'\x00', b'\x00', b'\x00', b'\x00', b'\x00',
    b'\x00', b'\x00', b'\x00', b'\x00', b'\x00',
];
pub const NOT_TOKEN_RANGES: &[u8; 16] = &[
    b'=', b'=', b'{', b'{', b'}', b'}', b'\x00', b'\x00', b'\x00', b'\x00', b'\x00', b'\x00',
    b'\x00', b'\x00', b'\x00', b'\x00',
];
pub const STRING_LITTERAL_CONTENT_RANGES: &[u8; 16] = &[
    b'"', b'"', b'\x00', b'\x00', b'\x00', b'\x00', b'\x00', b'\x00', b'\x00', b'\x00', b'\x00',
    b'\x00', b'\x00', b'\x00', b'\x00', b'\x00',
];

pub const IDENTIFIER_RANGES: &[u8; 16] = &[
    b'\x00', b'\x2d', b'\x2f', b'\x2f', b';', b'@', b'[', b'^', b'`', b'`', b'{', b'\xff', b'\x00',
    b'\x00', b'\x00', b'\x00',
];

const CHUNK_SIZE: usize = 16;

use nom::error::ParseError;

use super::Res;

pub fn take_while_simd<'a, Condition, Error: ParseError<&'a str>>(
    cond: Condition,
    ranges: &'static [u8; CHUNK_SIZE],
) -> impl Fn(&'a str) -> Res<&'a str, &'a str>
where
    Condition: Fn(char) -> bool,
{
    move |input: &'a str| {
        let condition = |c| cond(c);
        if input.len() == 0 {
            return Ok(("", ""));
        } else if input.len() >= CHUNK_SIZE {
            simd_loop16(input, ranges)
        } else {
            take_while(condition)(input)
        }
    }
}

fn simd_loop16<'a>(str: &'a str, ranges: &[u8; CHUNK_SIZE]) -> Res<&'a str, &'a str> {
    let start = str.as_ptr() as usize;
    let mut i = str.as_ptr() as usize;
    let ranges16 = unsafe { _mm_loadu_si128(ranges.as_ptr() as *const _) };
    let _ranges_len = ranges.len() as i32;
    loop {
        let s1 = unsafe { _mm_loadu_si128(i as *const _) };

        let idx = unsafe {
            _mm_cmpestri(
                ranges16,
                CHUNK_SIZE as i32,
                s1,
                CHUNK_SIZE as i32,
                _SIDD_LEAST_SIGNIFICANT | _SIDD_CMP_RANGES | _SIDD_UBYTE_OPS,
            )
        };

        if idx != CHUNK_SIZE as i32 {
            i += idx as usize;
            break;
        }
        i += CHUNK_SIZE;
    }
    let index = i - start;
    let (before, after) = str.split_at(min(index, str.len()));
    return Ok((after, before));
}

#[cfg(test)]
mod tests {
    use nom::error::VerboseError;

    use crate::clausewitz::tables::is_space;

    use super::*;
    #[test]
    fn take_while_simd__string_with_leading_whitespace__whitespace_collected_remainder_returned() {
        let text = " \t\n\r|Stop this is a big long string";
        let ranges = SPACE_RANGES;
        let (remainder, parsed) =
            take_while_simd::<'_, _, VerboseError<&str>>(is_space, ranges)(text).unwrap();
        assert_eq!(remainder, "|Stop this is a big long string");
        assert_eq!(parsed, " \t\n\r");
    }

    #[test]
    fn take_while_simd__16_character_string__whitespace_collected_remainder_returned() {
        let text = "1111111111111111";
        let ranges = SPACE_RANGES;
        let (remainder, parsed) =
            take_while_simd::<'_, _, VerboseError<&str>>(is_space, ranges)(text).unwrap();
        assert_eq!(remainder, "1111111111111111");
        assert_eq!(parsed, "");
    }

    #[test]
    fn take_while_simd__16_newlines_1_1__whitespace_collected_remainder_returned() {
        let text = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n1";
        let ranges = SPACE_RANGES;
        let (remainder, parsed) =
            take_while_simd::<'_, _, VerboseError<&str>>(is_space, ranges)(text).unwrap();
        assert_eq!(remainder, "1");
        assert_eq!(parsed, "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
    }

    #[test]
    fn take_while_simd__17_newlines_1_1__whitespace_collected_remainder_returned() {
        let text = "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n1";
        let ranges = SPACE_RANGES;
        let (remainder, parsed) =
            take_while_simd::<'_, _, VerboseError<&str>>(is_space, ranges)(text).unwrap();
        assert_eq!(remainder, "1");
        assert_eq!(parsed, "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
    }

    #[test]
    fn take_while_simd__string_with_many_leading_whitespace__whitespace_collected_remainder_returned(
    ) {
        let text = "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t|Stop this is a big long string";
        let ranges = SPACE_RANGES;
        let (remainder, parsed) =
            take_while_simd::<'_, _, VerboseError<&str>>(is_space, ranges)(text).unwrap();
        assert_eq!(remainder, "|Stop this is a big long string");
        assert_eq!(parsed, "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t");
    }

    #[test]
    fn take_while_simd__short_string__whitespace_collected_remainder_returned() {
        let text = "\t\t\ts";
        let ranges = SPACE_RANGES;
        let (remainder, parsed) =
            take_while_simd::<'_, _, VerboseError<&str>>(is_space, ranges)(text).unwrap();
        assert_eq!(remainder, "s");
        assert_eq!(parsed, "\t\t\t");
    }

    #[test]
    fn take_while_simd__all_white_space__whitespace_collected_remainder_returned() {
        let text = " \t\n\r";
        let ranges = SPACE_RANGES;
        let (remainder, parsed) =
            take_while_simd::<'_, _, VerboseError<&str>>(is_space, ranges)(text).unwrap();
        assert_eq!(remainder, "");
        assert_eq!(parsed, " \t\n\r");
    }
}
