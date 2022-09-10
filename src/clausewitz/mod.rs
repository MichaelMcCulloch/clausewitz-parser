use nom::{error::VerboseError, IResult};

#[cfg(test)]
pub(self) mod tests;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2"
))]
pub(crate) mod simd;

pub mod bracketed;
pub(crate) mod quoted;
pub mod root;
pub(crate) mod space;
pub(crate) mod tables;
pub(crate) mod unquoted;
pub(crate) mod value;

pub(crate) type Res<T, S> = IResult<T, S, VerboseError<T>>;
