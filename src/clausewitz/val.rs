use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use chrono::{Datelike, NaiveDate};
use serde::{
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
    Serialize, Serializer,
};

use crate::ClausewitzValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Val<'a> {
    Dict(Vec<(&'a str, Val<'a>)>),
    NumberedDict(i64, Vec<(&'a str, Val<'a>)>),
    Array(Vec<(u64, Val<'a>)>),
    Set(Vec<Val<'a>>),
    StringLiteral(&'a str),
    // #[serde(serialize_with = "serialize_naive_date")]
    Date(NaiveDate),
    Decimal(f64),
    Integer(i64),
    Identifier(&'a str),
}

impl<'a> Serialize for Val<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Val::Dict(seq) => serialize_dict(seq, serializer),
            Val::NumberedDict(n, seq) => {
                let mut tup = serializer.serialize_tuple(2)?;
                tup.serialize_element(n)?;
                tup.serialize_element(&Val::Dict(seq.clone()))?;
                tup.end()
            }
            Val::Array(arr) => serialize_array(arr, serializer),
            Val::Set(set) => serialize_set(set, serializer),
            Val::StringLiteral(str) => serializer.serialize_str(str),
            Val::Date(date) => serialize_naive_date(date, serializer),
            Val::Decimal(dec) => serializer.serialize_f64(*dec),
            Val::Integer(int) => serializer.serialize_i64(*int),
            Val::Identifier(id) => serializer.serialize_str(id),
        }
    }
}

fn serialize_dict<S>(dict: &Vec<(&str, Val)>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(dict.len()))?;
    for (k, v) in dict {
        map.serialize_entry(k, v)?;
    }
    map.end()
}
fn serialize_set<S>(set: &Vec<Val>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_tuple(set.len())?;
    for v in set {
        map.serialize_element(v)?;
    }
    map.end()
}
fn serialize_array<S>(arr: &Vec<(u64, Val)>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(arr.len()))?;
    for e in arr {
        seq.serialize_element(e)?;
    }
    seq.end()
}

pub fn serialize_naive_date<S>(date: &NaiveDate, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let val = (date.year(), date.month(), date.day());
    let mut state = ser.serialize_tuple(3)?;
    state.serialize_element(&val.0)?;
    state.serialize_element(&val.1)?;
    state.serialize_element(&val.2)?;
    state.end()
}

#[derive(Debug, PartialEq)]
pub struct IndexError {
    err: String,
}
impl Error for IndexError {}

impl Display for IndexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}

impl<'a> ClausewitzValue<'a> for Val<'a> {
    fn get_set_at_path<'b>(&'a self, path: &'b str) -> Result<&'a Vec<Val<'a>>, IndexError> {
        match self.get_at_path(path)? {
            Val::Set(s) => Ok(s),
            _ => Err(IndexError {
                err: format!("{} is not the set you are looking for!", path),
            }),
        }
    }
    fn get_date_at_path<'b>(&'a self, path: &'b str) -> Result<&'a NaiveDate, IndexError> {
        match self.get_at_path(path)? {
            Val::Date(d) => Ok(d),
            _ => Err(IndexError {
                err: format!("{} is not the date you are looking for!", path),
            }),
        }
    }
    fn get_string_at_path<'b>(&'a self, path: &'b str) -> Result<&'a str, IndexError> {
        match self.get_at_path(path)? {
            Val::StringLiteral(s) => Ok(s),
            _ => Err(IndexError {
                err: format!("{} is not the string you are looking for!", path),
            }),
        }
    }
    fn get_identifier_at_path<'b>(&'a self, path: &'b str) -> Result<&'a str, IndexError> {
        match self.get_at_path(path)? {
            Val::Identifier(s) => Ok(s),
            _ => Err(IndexError {
                err: format!("{} is not the identifier you are looking for!", path),
            }),
        }
    }
    fn get_decimal_at_path<'b>(&'a self, path: &'b str) -> Result<&'a f64, IndexError> {
        match self.get_at_path(path)? {
            Val::Decimal(f) => Ok(f),
            _ => Err(IndexError {
                err: format!("{} is not the decimal you are looking for!", path),
            }),
        }
    }
    fn get_integer_at_path<'b>(&'a self, path: &'b str) -> Result<&'a i64, IndexError> {
        match self.get_at_path(path)? {
            Val::Integer(f) => Ok(f),
            _ => Err(IndexError {
                err: format!("{} is not the integer you are looking for!", path),
            }),
        }
    }

    fn get_number_at_path<'b>(&'a self, path: &'b str) -> Result<f64, IndexError> {
        match self.get_at_path(path)? {
            Val::Integer(f) => Ok(*f as f64),
            Val::Decimal(f) => Ok(*f),
            _ => Err(IndexError {
                err: format!(
                    "{} is not the integer or decimal you are looking for!",
                    path
                ),
            }),
        }
    }
    fn get_array_at_path<'b>(
        &'a self,
        path: &'b str,
    ) -> Result<&'a Vec<(u64, Val<'a>)>, IndexError> {
        match self.get_at_path(path)? {
            Val::Array(v) => Ok(v),
            _ => Err(IndexError {
                err: format!("{} is not the array you are looking for!", path),
            }),
        }
    }
    fn get_dict_at_path<'b>(
        &'a self,
        path: &'b str,
    ) -> Result<&'a Vec<(&'a str, Val<'a>)>, IndexError> {
        match self.get_at_path(path)? {
            Val::Dict(v) => Ok(v),
            _ => Err(IndexError {
                err: format!("{} is not the dict you are looking for!", path),
            }),
        }
    }
    fn get_numbered_dict_at_path<'b>(
        &'a self,
        path: &'b str,
    ) -> Result<(&'a i64, &'a Vec<(&'a str, Val<'a>)>), IndexError> {
        match self.get_at_path(path)? {
            Val::NumberedDict(n, v) => Ok((n, v)),
            _ => Err(IndexError {
                err: format!("{} is not the numbered dict you are looking for!", path),
            }),
        }
    }

    fn get_at_path<'b>(&'a self, path: &'b str) -> Result<&'a Val<'a>, IndexError> {
        let path_components = path.split(".").collect::<Vec<_>>();
        path_components
            .into_iter()
            .fold(Ok(self), |result, p| match result {
                Ok(Val::Dict(dict_inner)) => {
                    let filtered_values = dict_inner
                        .iter()
                        .filter_map(|(k, v)| if k == &p { Some(v) } else { None })
                        .collect::<Vec<_>>();
                    let val_for_key = filtered_values.first();

                    match val_for_key {
                        Some(val) => Ok(val),
                        None => Err(IndexError {
                            err: format!("Expected to find value with key {}", p),
                        }),
                    }
                }

                Ok(Val::NumberedDict(_number, num_dict_inner)) => {
                    let filtered_values = num_dict_inner
                        .iter()
                        .filter_map(|(k, v)| if k == &p { Some(v) } else { None })
                        .collect::<Vec<_>>();
                    let dict_value = filtered_values.first();
                    match dict_value {
                        Some(val) => Ok(val),
                        None => Err(IndexError {
                            err: format!("Expected to find value with key {}", p),
                        }),
                    }
                }

                Ok(Val::Array(vec)) => {
                    let index = p.parse::<u64>().unwrap();
                    let element = vec
                        .iter()
                        .find_map(|(i, v)| if i == &index { Some(v) } else { None });
                    match element {
                        Some(val) => Ok(&val),
                        None => Err(IndexError {
                            err: format!("Expected to find value with index {}", p),
                        }),
                    }
                }
                Ok(Val::Set(_)) => Err(IndexError {
                    err: format!("Cannot index a set with index {}", p),
                }),
                Err(e) => Err(e),
                _ => Err(IndexError {
                    err: format!("Cannot index terminal values!"),
                }),
            })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test____() {
        let s = std::fs::read_to_string(
            "/home/michael/Dev/Stellarust/stellarust5/production_data/gamestate",
        )
        .unwrap();
        let gamestate = crate::root(&s).unwrap().1;
        print!(
            "{}",
            serde_json::to_string_pretty(gamestate.get_at_path("country.1").unwrap()).unwrap()
        )
    }
    #[test]
    fn val_dict__given_key__returns_val_result() {
        let val = Val::Dict(vec![("key", Val::Integer(10))]);
        let index = "key";

        let dict_val = val.get_at_path(index);

        assert_eq!(Ok(&Val::Integer(10)), dict_val);
    }
    #[test]
    fn val_numbered_dict__given_key__returns_val_result() {
        let val = Val::NumberedDict(0, vec![("key", Val::Integer(10))]);
        let index = "key";

        let dict_val = val.get_at_path(index);

        assert_eq!(Ok(&Val::Integer(10)), dict_val);
    }
    #[test]
    fn val_array__given_index__returns_val_result() {
        let val = Val::Array(vec![(0, Val::Integer(10))]);
        let index = "0";

        let dict_val = val.get_at_path(index);

        assert_eq!(Ok(&Val::Integer(10)), dict_val);
    }

    #[test]
    fn val_array_of_dicts__given_index_dot_key__returns_val_result() {
        let val = Val::Array(vec![(
            0,
            Val::Dict(vec![("key", Val::StringLiteral("value"))]),
        )]);
        let index = "0.key";

        let string_literal_val = val.get_at_path(index);

        assert_eq!(Ok(&Val::StringLiteral("value")), string_literal_val);
    }

    #[test]
    fn val_dict_of_arrays__given_key_dot_index__returns_val_result() {
        let val = Val::Dict(vec![(
            "key",
            Val::Array(vec![(0, Val::StringLiteral("value"))]),
        )]);
        let index = "key.0";

        let string_literal_val = val.get_at_path(index);

        assert_eq!(Ok(&Val::StringLiteral("value")), string_literal_val);
    }
}
