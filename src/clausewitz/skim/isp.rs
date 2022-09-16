use std::{
    borrow::Borrow,
    ops::{RangeFrom, RangeTo},
    str::{CharIndices, Chars},
};

use chrono::ParseError;
use nom::{InputIter, InputLength, InputTake, InputTakeAtPosition, Needed, Offset, Slice};

#[derive(Clone, Copy, Debug)]
pub struct InputSearchPair<'a, 'b> {
    pub slice: &'a str,
    pub search_path: [&'b str; 10],
    pub search_path_index: usize,
}
impl<'a, 'b> Offset for InputSearchPair<'a, 'b> {
    fn offset(&self, second: &Self) -> usize {
        self.slice.offset(second.slice)
    }
}
impl<'a, 'b> InputTake for InputSearchPair<'a, 'b> {
    fn take(&self, count: usize) -> Self {
        Self {
            slice: self.slice.take(count),
            search_path: self.search_path,
            search_path_index: self.search_path_index,
        }
    }

    // return byte index
    fn take_split(&self, count: usize) -> (Self, Self) {
        let (a, b) = self.slice.take_split(count);
        (
            Self {
                slice: a,
                search_path: self.search_path,
                search_path_index: self.search_path_index,
            },
            Self {
                slice: b,
                search_path: self.search_path,
                search_path_index: self.search_path_index,
            },
        )
    }
}

impl<'a, 'b> nom::error::ParseError<&'a str> for InputSearchPair<'a, 'b> {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        Self {
            slice: input,
            search_path: [""; 10],
            search_path_index: 0,
        }
    }

    fn append(input: &str, kind: nom::error::ErrorKind, other: Self) -> Self {
        Self {
            slice: other.slice,
            search_path: other.search_path,
            search_path_index: other.search_path_index,
        }
    }
}

impl<'a, 'b> InputTakeAtPosition for InputSearchPair<'a, 'b> {
    type Item = char;

    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.slice.find(predicate) {
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => unsafe {
                Ok((
                    Self {
                        slice: self.slice.get_unchecked(i..),
                        search_path: self.search_path,
                        search_path_index: self.search_path_index,
                    },
                    Self {
                        slice: self.slice.get_unchecked(..i),
                        search_path: self.search_path,
                        search_path_index: self.search_path_index,
                    },
                ))
            },
            None => Err(nom::Err::Incomplete(Needed::new(1))),
        }
    }

    fn split_at_position1<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.slice.find(predicate) {
            Some(0) => Err(nom::Err::Error(E::from_error_kind(*self, e))),
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => unsafe {
                Ok((
                    Self {
                        slice: self.slice.get_unchecked(i..),
                        search_path: self.search_path,
                        search_path_index: self.search_path_index,
                    },
                    Self {
                        slice: self.slice.get_unchecked(..i),
                        search_path: self.search_path,
                        search_path_index: self.search_path_index,
                    },
                ))
            },
            None => Err(nom::Err::Incomplete(Needed::new(1))),
        }
    }

    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.slice.find(predicate) {
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => unsafe {
                Ok((
                    Self {
                        slice: self.slice.get_unchecked(i..),
                        search_path: self.search_path,
                        search_path_index: self.search_path_index,
                    },
                    Self {
                        slice: self.slice.get_unchecked(..i),
                        search_path: self.search_path,
                        search_path_index: self.search_path_index,
                    },
                ))
            },
            // the end of slice is a char boundary
            None => unsafe {
                Ok((
                    Self {
                        slice: self.slice.get_unchecked(self.slice.len()..),
                        search_path: self.search_path,
                        search_path_index: self.search_path_index,
                    },
                    Self {
                        slice: self.slice.get_unchecked(..self.slice.len()),
                        search_path: self.search_path,
                        search_path_index: self.search_path_index,
                    },
                ))
            },
        }
    }

    fn split_at_position1_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.slice.find(predicate) {
            Some(0) => Err(nom::Err::Error(E::from_error_kind(*self, e))),
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => unsafe {
                Ok((
                    Self {
                        slice: self.slice.get_unchecked(i..),
                        search_path: self.search_path,
                        search_path_index: self.search_path_index,
                    },
                    Self {
                        slice: self.slice.get_unchecked(..i),
                        search_path: self.search_path,
                        search_path_index: self.search_path_index,
                    },
                ))
            },
            None => {
                if self.slice.is_empty() {
                    Err(nom::Err::Error(E::from_error_kind(*self, e)))
                } else {
                    // the end of slice is a char boundary
                    unsafe {
                        Ok((
                            Self {
                                slice: self.slice.get_unchecked(self.slice.len()..),
                                search_path: self.search_path,
                                search_path_index: self.search_path_index,
                            },
                            Self {
                                slice: self.slice.get_unchecked(..self.slice.len()),
                                search_path: self.search_path,
                                search_path_index: self.search_path_index,
                            },
                        ))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use nom::bytes::complete::take_while;

    use crate::clausewitz::skim::SkimResult;

    use super::*;
    #[test]
    fn test_name() {
        let search = InputSearchPair::create("asdffdsa", "asdf");

        let x: SkimResult<InputSearchPair, InputSearchPair> = take_while(|f| f != 'f')(search);
        println!("{:?}", search);
        println!("{:?}", x);
    }
}
impl<'a, 'b> InputSearchPair<'a, 'b> {
    pub fn create(input: &'a str, search: &'b str) -> Self {
        let mut v = [""; 10];
        let mut vec = search.split('.').collect::<Vec<_>>();
        vec.reverse();
        for i in 0..vec.len() {
            v[i] = vec[i]
        }
        v.reverse();

        InputSearchPair {
            slice: input,
            search_path: v,
            search_path_index: 10 - vec.len(),
        }
    }
}
impl<'a, 'b> Borrow<str> for InputSearchPair<'a, 'b> {
    fn borrow(&self) -> &str {
        self.slice.borrow()
    }
}
impl<'a, 'b> Slice<RangeFrom<usize>> for InputSearchPair<'a, 'b> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        Self {
            slice: self.slice.slice(range),
            search_path: self.search_path,
            search_path_index: self.search_path_index,
        }
    }
}
impl<'a, 'b> Slice<RangeTo<usize>> for InputSearchPair<'a, 'b> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        Self {
            slice: self.slice.slice(range),
            search_path: self.search_path,
            search_path_index: self.search_path_index,
        }
    }
}
impl<'a, 'b> InputLength for InputSearchPair<'a, 'b> {
    fn input_len(&self) -> usize {
        self.slice.input_len()
    }
}

impl<'a, 'b> InputIter for InputSearchPair<'a, 'b> {
    type Item = char;
    type Iter = CharIndices<'a>;
    type IterElem = Chars<'a>;
    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        self.slice.char_indices()
    }
    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.slice.chars()
    }
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.slice.position(predicate)
    }
    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        self.slice.slice_index(count)
    }
}
