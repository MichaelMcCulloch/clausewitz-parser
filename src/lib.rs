mod clausewitz;

use chrono::NaiveDate;
pub use clausewitz::{
    bracketed::key_value,
    root::{cheat_root, root},
    skim,
    val::{IndexError, Val},
};

pub trait ClausewitzValue<'a> {
    fn get_set_at_path<'b>(&'a self, path: &'b str) -> Result<&'a Vec<Val<'a>>, IndexError>;
    fn get_date_at_path<'b>(&'a self, path: &'b str) -> Result<&'a NaiveDate, IndexError>;
    fn get_string_at_path<'b>(&'a self, path: &'b str) -> Result<&'a str, IndexError>;
    fn get_identifier_at_path<'b>(&'a self, path: &'b str) -> Result<&'a str, IndexError>;
    fn get_decimal_at_path<'b>(&'a self, path: &'b str) -> Result<&'a f64, IndexError>;
    fn get_integer_at_path<'b>(&'a self, path: &'b str) -> Result<&'a i64, IndexError>;
    fn get_number_at_path<'b>(&'a self, path: &'b str) -> Result<f64, IndexError>;
    fn get_array_at_path<'b>(
        &'a self,
        path: &'b str,
    ) -> Result<&'a Vec<(u64, Val<'a>)>, IndexError>;
    fn get_dict_at_path<'b>(
        &'a self,
        path: &'b str,
    ) -> Result<&'a Vec<(&'a str, Val<'a>)>, IndexError>;
    fn get_numbered_dict_at_path<'b>(
        &'a self,
        path: &'b str,
    ) -> Result<(&'a i64, &'a Vec<(&'a str, Val<'a>)>), IndexError>;
    fn get_at_path<'b>(&'a self, path: &'b str) -> Result<&'a Val<'a>, IndexError>;
}
