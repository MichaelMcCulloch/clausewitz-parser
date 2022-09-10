use crate::val::Val;

use super::{bracketed::bracketed, quoted::quoted, unquoted::unquoted, Res};
use nom::branch::alt;

pub fn value<'a>(input: &'a str) -> Res<&'a str, Val<'a>> {
    alt((bracketed, quoted, unquoted))(input)
}
