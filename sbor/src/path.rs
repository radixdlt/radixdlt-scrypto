use crate::any::Fields;
use crate::any::Value;

pub struct SborRelPath<'a>(&'a [usize]);

impl<'a> SborRelPath<'a> {
    pub fn new(path: &'a [usize]) -> Self {
        SborRelPath(path)
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn pop(&self) -> (usize, Self) {
        let (index_slice, extended_path) = self.0.split_at(1);
        let index = index_slice[0];
        (index, SborRelPath(extended_path))
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

    pub fn get_from(self, value: &'a Value) -> Option<&'a Value> {
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
