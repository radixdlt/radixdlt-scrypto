use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::value::Value;
use crate::CustomValueKind;

#[derive(Eq, PartialEq, Clone)]
pub struct SborPathBuf(Vec<usize>);

impl SborPathBuf {
    pub fn new() -> Self {
        SborPathBuf(vec![])
    }

    pub fn push(&mut self, path: usize) {
        self.0.push(path);
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }
}

impl From<SborPathBuf> for SborPath {
    fn from(mutable: SborPathBuf) -> Self {
        SborPath::new(mutable.0)
    }
}

/// A series of indexes which describes some value in the sbor tree
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct SborPath(Vec<usize>);

impl SborPath {
    pub fn new(path: Vec<usize>) -> Self {
        SborPath(path)
    }

    pub fn get_from_value<'a, X: CustomValueKind, Y>(
        &'a self,
        value: &'a Value<X, Y>,
    ) -> Option<&'a Value<X, Y>> {
        let rel_path = SborValueRetriever(&self.0);
        rel_path.get_from(value)
    }

    pub fn get_from_value_mut<'a, X: CustomValueKind, Y>(
        &'a self,
        value: &'a mut Value<X, Y>,
    ) -> Option<&'a mut Value<X, Y>> {
        let rel_path = SborValueRetriever(&self.0);
        rel_path.get_from_mut(value)
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

    fn get_from_vector<X: CustomValueKind, Y>(
        &self,
        values: &'a [Value<X, Y>],
    ) -> Option<&'a Value<X, Y>> {
        let (index, next_path) = self.pop();
        values
            .get(index)
            .and_then(|value| next_path.get_from(value))
    }

    fn get_from<X: CustomValueKind, Y>(self, value: &'a Value<X, Y>) -> Option<&'a Value<X, Y>> {
        if self.is_empty() {
            return Option::Some(value);
        }

        match value {
            Value::Tuple { fields: vec, .. }
            | Value::Enum { fields: vec, .. }
            | Value::Array { elements: vec, .. } => self.get_from_vector(vec),
            _ => Option::None,
        }
    }

    fn get_from_vector_mut<X: CustomValueKind, Y>(
        &self,
        values: &'a mut [Value<X, Y>],
    ) -> Option<&'a mut Value<X, Y>> {
        let (index, next_path) = self.pop();
        values
            .get_mut(index)
            .and_then(|value| next_path.get_from_mut(value))
    }

    fn get_from_mut<X: CustomValueKind, Y>(
        self,
        value: &'a mut Value<X, Y>,
    ) -> Option<&'a mut Value<X, Y>> {
        if self.is_empty() {
            return Option::Some(value);
        }

        match value {
            Value::Tuple { fields: vec, .. }
            | Value::Enum { fields: vec, .. }
            | Value::Array { elements: vec, .. } => self.get_from_vector_mut(vec),
            _ => Option::None,
        }
    }
}
