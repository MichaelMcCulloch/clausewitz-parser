use std::fmt::Debug;

use crate::clausewitz::Res;

pub fn assert_result_ok<T: Debug + Clone>(result: Res<&str, T>) {
    let result2 = result.clone();
    if result2.is_err() {
        match result2.clone().err().unwrap() {
            nom::Err::Incomplete(e) => println!("{:#?}", e),
            nom::Err::Error(e) => println!("{:#?}", e),
            nom::Err::Failure(e) => println!("{:#?}", e),
        };
    }
    assert!(result.is_ok());
    assert!(result.unwrap().0.is_empty())
}
