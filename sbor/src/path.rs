use crate::any::Fields;
use crate::any::Value;
use crate::rust::vec::Vec;
use sbor::*;

/// A series of indexes which describes some value in the sbor tree
pub struct SborPath(Vec<usize>);

impl SborPath {
    pub fn new(path: Vec<usize>) -> Self {
        SborPath(path)
    }

    pub fn get_from_value<'a>(&'a self, value: &'a Value) -> Option<&'a Value> {
        let rel_path = SborValueRetriever(&self.0);
        rel_path.get_from(value)
    }
}

/// Helper structure which helps in retrieving a value given a root value and sbor path
struct SborValueRetriever<'a>(&'a [usize]);

impl<'a> SborValueRetriever<'a> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn pop(&self) -> (usize, Self) {
        let (index_slice, extended_path) = self.0.split_at(1);
        let index = index_slice[0];
        (index, SborValueRetriever(extended_path))
    }

    fn get_from_vector(&self, values: &'a [Value]) -> Option<&'a Value> {
        let (index, next_path) = self.pop();
        values
            .get(index)
            .and_then(|value| next_path.get_from(value))
    }

    fn get_from_fields(&self, fields: &'a Fields) -> Option<&'a Value> {
        match fields {
            Fields::Named(values) | Fields::Unnamed(values) => self.get_from_vector(values),
            Fields::Unit => Option::None,
        }
    }

    fn get_from(self, value: &'a Value) -> Option<&'a Value> {
        if self.is_empty() {
            return Option::Some(value);
        }

        match value {
            Value::Struct(fields) | Value::Enum(_, fields) => self.get_from_fields(fields),
            Value::Array(_, values) | Value::Vec(_, values) => self.get_from_vector(values),
            _ => Option::None,
        }
    }
}
