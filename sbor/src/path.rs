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

    pub fn get_from(self, value: &'a Value) -> Option<&'a Value> {
        if self.is_empty() {
            return Option::Some(value);
        }

        match value {
            Value::Struct(values)
            | Value::Enum(_, values)
            | Value::Array(_, values)
            | Value::Vec(_, values) => self.get_from_vector(values),
            _ => Option::None,
        }
    }
}
