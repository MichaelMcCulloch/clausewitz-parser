use std::fmt::Display;

use chrono::NaiveDate;
use nom::{error::VerboseError, IResult};

#[cfg(test)]
pub(self) mod tests;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2"
))]
pub(crate) mod simd;

pub(crate) mod bracketed;
pub(crate) mod quoted;
pub mod root;
pub(crate) mod space;
pub(crate) mod tables;
pub(crate) mod unquoted;
pub(crate) mod value;

pub(crate) type Res<T, S> = IResult<T, S, VerboseError<T>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Val<'a> {
    Dict(Vec<(&'a str, Val<'a>)>),
    NumberedDict(i64, Vec<(&'a str, Val<'a>)>),
    Array(Vec<Val<'a>>),
    Set(Vec<Val<'a>>),
    StringLiteral(&'a str),
    Date(NaiveDate),
    Decimal(f64),
    Integer(i64),
    Identifier(&'a str),
}

impl Display for Val<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            Val::Dict(_dict_entries) => format_dict(self, ""),
            Val::NumberedDict(_number, _dict_entries) => format_numbered_dict(self, ""),
            Val::Array(_array_elements) => format_array(self, ""),
            Val::Set(_set_elements) => format_set(self, ""),
            Val::StringLiteral(_string) => format_string_literal(self, ""),
            Val::Date(_date) => format_date(self, ""),
            Val::Decimal(_decimal) => format_decimal(self, ""),
            Val::Integer(_integer) => format_integer(self, ""),
            Val::Identifier(_identifire) => format_identifier(self, ""),
        };

        write!(f, "{}", result)
    }
}

fn format_val(val: &Val, tabs: &str) -> String {
    match val {
        Val::Dict(_) => format_dict(val, tabs),
        Val::NumberedDict(_, _) => format_numbered_dict(val, tabs),
        Val::Array(_) => format_array(val, tabs),
        Val::Set(_) => format_set(val, tabs),
        Val::StringLiteral(_) => format_string_literal(val, tabs),
        Val::Date(_) => format_date(val, tabs),
        Val::Decimal(_) => format_decimal(val, tabs),
        Val::Integer(_) => format_integer(val, tabs),
        Val::Identifier(_) => format_identifier(val, tabs),
    }
}

fn format_dict(val: &Val, tabs: &str) -> String {
    match val {
        Val::Dict(dict_entries) => {
            let mut elements_strings: Vec<String> = vec![];

            for (key, value) in dict_entries {
                elements_strings.push(get_key_value_pair_strings(value, tabs, key));
            }
            format!("{}(\n{}\n{})", tabs, elements_strings.join("\n"), tabs)
        }

        _ => panic!("whoops"),
    }
}

fn get_key_value_pair_strings(value: &Val, tabs: &str, key: &&str) -> String {
    match value {
        Val::Dict(_) => format!(
            "{}    {} = \n{}",
            tabs,
            key,
            format_val(&value, &format!("{}{}", tabs, "      "))
        ),
        Val::NumberedDict(_, _) => format!(
            "{}    {} = \n{}",
            tabs,
            key,
            format_val(&value, &format!("{}{}", tabs, "      "))
        ),
        Val::Array(_) => format!(
            "{}    {} = \n{}",
            tabs,
            key,
            format_val(&value, &format!("{}{}", tabs, "      "))
        ),
        Val::Set(_) => format!(
            "{}    {} = \n{}",
            tabs,
            key,
            format_val(&value, &format!("{}{}", tabs, "      "))
        ),
        Val::StringLiteral(_) => {
            format!("{}    {} = {}", tabs, key, format_val(&value, ""))
        }
        Val::Date(_) => {
            format!("{}    {} = {}", tabs, key, format_val(&value, ""))
        }
        Val::Decimal(_) => {
            format!("{}    {} = {}", tabs, key, format_val(&value, ""))
        }
        Val::Integer(_) => {
            format!("{}    {} = {}", tabs, key, format_val(&value, ""))
        }
        Val::Identifier(_) => {
            format!("{}    {} = {}", tabs, key, format_val(&value, ""))
        }
    }
}
fn format_numbered_dict(val: &Val, tabs: &str) -> String {
    match val {
        Val::NumberedDict(number, dict_entries) => {
            let mut elements_strings: Vec<String> = vec![];

            for (key, value) in dict_entries {
                elements_strings.push(get_key_value_pair_strings(value, tabs, key));
            }
            format!(
                "{}[{}] (\n{}\n{})",
                tabs,
                number,
                elements_strings.join("\n"),
                tabs
            )
        }

        _ => panic!("whoops"),
    }
}
fn format_array(val: &Val, tabs: &str) -> String {
    match val {
        Val::Array(array_elements) => {
            let mut elements_strings: Vec<String> = vec![];

            for element in array_elements {
                elements_strings.push(format_val(element, &format!("{}{}", tabs, "    ")));
            }
            format!("{}[\n{}\n{}]", tabs, elements_strings.join("\n"), tabs)
        }
        _ => panic!("whoops"),
    }
}
fn format_set(val: &Val, tabs: &str) -> String {
    match val {
        Val::Set(set_elements) => {
            let mut elements_strings: Vec<String> = vec![];

            for element in set_elements {
                elements_strings.push(format_val(element, &format!("{}{}", tabs, "    ")));
            }
            format!("{}{{\n{}\n{}}}", tabs, elements_strings.join("\n"), tabs)
        }
        _ => panic!("whoops"),
    }
}
fn format_string_literal(val: &Val, tabs: &str) -> String {
    match val {
        Val::StringLiteral(string) => format!("{}\"{}\"", tabs, string),
        _ => panic!("whoops"),
    }
}
fn format_date(val: &Val, tabs: &str) -> String {
    match val {
        Val::Date(date) => format!("{}{}", tabs, date),
        _ => panic!("whoops"),
    }
}
fn format_decimal(val: &Val, tabs: &str) -> String {
    match val {
        Val::Decimal(decimal) => format!("{}{}", tabs, decimal),
        _ => panic!("whoops"),
    }
}
fn format_integer(val: &Val, tabs: &str) -> String {
    match val {
        Val::Integer(integer) => format!("{}{}", tabs, integer),
        _ => panic!("whoops"),
    }
}
fn format_identifier(val: &Val, tabs: &str) -> String {
    match val {
        Val::Identifier(identifire) => {
            format!("{}{}", tabs, identifire)
        }
        _ => panic!("whoops"),
    }
}

#[cfg(test)]
mod format_tests {
    use chrono::NaiveDate;

    use crate::Val;

    #[test]
    fn format_integer() {
        let _str = format!("{}", Val::Integer(0));
    }

    #[test]
    fn format_decimal() {
        let _str = format!("{}", Val::Decimal(0.0));
    }

    #[test]
    fn format_identifier() {
        let _str = format!("{}", Val::Identifier("identifier"));
    }

    #[test]
    fn format_string_literal() {
        let _str = format!("{}", Val::StringLiteral("String Litteral"));
    }

    #[test]
    fn format_date() {
        let _str = format!("{}", Val::Date(NaiveDate::from_ymd(2021, 1, 1)));
    }

    #[test]
    fn format_set() {
        let _str = format!(
            "{}",
            Val::Set(vec![Val::Integer(0), Val::Set(vec![Val::Integer(0)])])
        );
    }

    #[test]
    fn format_dict() {
        let _str = format!(
            "{}",
            Val::Dict(vec![
                ("key", Val::Integer(0)),
                ("dict", Val::Dict(vec![("key", Val::Integer(0))]))
            ])
        );
    }

    #[test]
    fn format_dict2() {
        let _str = format!("{}", Val::Dict(vec![("key", Val::Integer(0)),]));
    }

    #[test]
    fn format_NumberedDict() {
        let _str = format!(
            "{}",
            Val::NumberedDict(
                0,
                vec![
                    ("key", Val::Integer(0)),
                    (
                        "NumberedDict",
                        Val::NumberedDict(1, vec![("key", Val::Integer(0))])
                    )
                ]
            )
        );
    }

    #[test]
    fn format_NumberedDict2() {
        let _str = format!(
            "{}",
            Val::NumberedDict(-234, vec![("key", Val::Integer(0)),])
        );
    }
}
