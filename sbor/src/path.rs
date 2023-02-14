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

impl From<SborPath> for SborPathBuf {
    fn from(path: SborPath) -> Self {
        Self(path.0)
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
        let rel_path = ValueRetriever(&self.0);
        rel_path.get_from(value)
    }

    pub fn get_from_value_mut<'a, X: CustomValueKind, Y>(
        &'a self,
        value: &'a mut Value<X, Y>,
    ) -> Option<&'a mut Value<X, Y>> {
        let rel_path = ValueRetriever(&self.0);
        rel_path.get_from_mut(value)
    }
}

/// Helper structure which helps in retrieving a value given a root value and sbor path
struct ValueRetriever<'a>(&'a [usize]);

impl<'a> ValueRetriever<'a> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn advance(&self) -> Option<(usize, Self)> {
        if self.is_empty() {
            return None;
        }

        let (index_slice, extended_path) = self.0.split_at(1);
        let index = index_slice[0];
        Some((index, ValueRetriever(extended_path)))
    }

    fn get_from<X: CustomValueKind, Y>(self, value: &'a Value<X, Y>) -> Option<&'a Value<X, Y>> {
        if self.is_empty() {
            return Option::Some(value);
        }

        match value {
            Value::Tuple { fields: v, .. }
            | Value::Enum { fields: v, .. }
            | Value::Array { elements: v, .. } => {
                let (index, next_path) = self.advance().expect("Should be available");
                v.get(index).and_then(|value| next_path.get_from(value))
            }
            Value::Map { entries, .. } => {
                let (index, next_path) = self.advance().expect("Should be available");
                entries.get(index).and_then(|value| {
                    if let Some((index, next_path)) = next_path.advance() {
                        match index {
                            0 => next_path.get_from(&value.0),
                            1 => next_path.get_from(&value.1),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
            }
            _ => Option::None,
        }
    }

    fn get_from_mut<X: CustomValueKind, Y>(
        self,
        value: &'a mut Value<X, Y>,
    ) -> Option<&'a mut Value<X, Y>> {
        if self.is_empty() {
            return Option::Some(value);
        }

        match value {
            Value::Tuple { fields: v, .. }
            | Value::Enum { fields: v, .. }
            | Value::Array { elements: v, .. } => {
                let (index, next_path) = self.advance().expect("Should be available");
                v.get_mut(index)
                    .and_then(|value| next_path.get_from_mut(value))
            }
            Value::Map { entries, .. } => {
                let (index, next_path) = self.advance().expect("Should be available");
                entries.get_mut(index).and_then(|value| {
                    if let Some((index, next_path)) = next_path.advance() {
                        match index {
                            0 => next_path.get_from_mut(&mut value.0),
                            1 => next_path.get_from_mut(&mut value.1),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
            }
            _ => Option::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::*;

    #[test]
    fn query_array() {
        let value = BasicValue::Array {
            element_value_kind: BasicValueKind::Array,
            elements: vec![BasicValue::Array {
                element_value_kind: BasicValueKind::U8,
                elements: vec![BasicValue::U8 { value: 5 }],
            }],
        };
        assert_eq!(
            SborPath(vec![0]).get_from_value(&value),
            Some(&BasicValue::Array {
                element_value_kind: BasicValueKind::U8,
                elements: vec![BasicValue::U8 { value: 5 }],
            })
        );
        assert_eq!(
            SborPath(vec![0, 0]).get_from_value(&value),
            Some(&BasicValue::U8 { value: 5 })
        );
        assert_eq!(SborPath(vec![0, 0, 0]).get_from_value(&value), None);
        assert_eq!(SborPath(vec![1]).get_from_value(&value), None);
        assert_eq!(SborPath(vec![0, 1]).get_from_value(&value), None);
        assert_eq!(SborPath(vec![0, 0, 1]).get_from_value(&value), None);
    }

    #[test]
    fn query_map() {
        let value = BasicValue::Map {
            key_value_kind: BasicValueKind::U8,
            value_value_kind: BasicValueKind::Array,
            entries: vec![(
                BasicValue::U8 { value: 3 },
                BasicValue::Array {
                    element_value_kind: BasicValueKind::U8,
                    elements: vec![BasicValue::U8 { value: 5 }],
                },
            )],
        };
        assert_eq!(
            SborPath(vec![0, 0]).get_from_value(&value),
            Some(&BasicValue::U8 { value: 3 })
        );
        assert_eq!(
            SborPath(vec![0, 1]).get_from_value(&value),
            Some(&BasicValue::Array {
                element_value_kind: BasicValueKind::U8,
                elements: vec![BasicValue::U8 { value: 5 }],
            })
        );
        assert_eq!(
            SborPath(vec![0, 1, 0]).get_from_value(&value),
            Some(&BasicValue::U8 { value: 5 })
        );

        assert_eq!(SborPath(vec![0]).get_from_value(&value), None);
        assert_eq!(SborPath(vec![0, 2]).get_from_value(&value), None);
        assert_eq!(SborPath(vec![0, 0, 0]).get_from_value(&value), None);
    }
}
