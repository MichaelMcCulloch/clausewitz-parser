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

use nom::error::{ParseError, VerboseError};

use super::tables::{is_identifier_char, is_space, is_string_litteral_contents, is_token};
use super::Res;

#[inline(always)]
pub fn take_simd_identifier<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    take_while_simd::<'a, _, VerboseError<&'a str>>(is_identifier_char, IDENTIFIER_RANGES)(input)
}

#[inline(always)]
pub fn take_simd_string_literal<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    take_while_simd::<'a, _, VerboseError<&'a str>>(
        is_string_litteral_contents,
        STRING_LITTERAL_CONTENT_RANGES,
    )(input)
}

#[inline(always)]
pub fn take_simd_not_token<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    take_while_simd::<'a, _, VerboseError<&'a str>>(
        move |character| !is_token(character),
        NOT_TOKEN_RANGES,
    )(input)
}

#[inline(always)]
pub fn take_simd_space<'a>(input: &'a str) -> Res<&'a str, &'a str> {
    take_while_simd::<'a, _, VerboseError<&'a str>>(
        move |character| is_space(character),
        SPACE_RANGES,
    )(input)
}

#[inline(always)]
fn take_while_simd<'a, Condition, Error: ParseError<&'a str>>(
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
#[inline(always)]

fn simd_loop16<'a>(string: &'a str, character_ranges: &[u8; CHUNK_SIZE]) -> Res<&'a str, &'a str> {
    // Get the starting pointer of the string
    let start_pointer = string.as_ptr() as usize;
    // Set the current pointer to the starting pointer
    let mut current_pointer = string.as_ptr() as usize;
    // Load the range of characters into a SIMD register
    let character_ranges16 = unsafe { _mm_loadu_si128(character_ranges.as_ptr() as *const _) };
    // Get the length of the range of characters
    let character_ranges_length = character_ranges.len() as i32;
    loop {
        // Load 16 bytes from the current pointer into a SIMD register
        let simd_register1 = unsafe { _mm_loadu_si128(current_pointer as *const _) };

        // Compare the range of characters with the 16 bytes loaded into the SIMD register
        let index = unsafe {
            _mm_cmpestri(
                character_ranges16,
                CHUNK_SIZE as i32,
                simd_register1,
                CHUNK_SIZE as i32,
                _SIDD_LEAST_SIGNIFICANT | _SIDD_CMP_RANGES | _SIDD_UBYTE_OPS,
            )
        };

        // If a character is found within the range, break out of the loop and get its index
        if index != CHUNK_SIZE as i32 {
            current_pointer += index as usize;
            break;
        }

        // Otherwise, move to the next 16 bytes
        current_pointer += CHUNK_SIZE;
    }

    // Calculate the index of the found character
    let character_index = current_pointer - start_pointer;

    // Split the string at the index and return a tuple containing the substring after and before it
    let (substring_before, substring_after) = string.split_at(min(character_index, string.len()));

    return Ok((substring_after, substring_before));
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
