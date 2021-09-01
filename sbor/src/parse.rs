#[cfg(any(feature = "serde_std", feature = "serde_alloc"))]
use serde::{Deserialize, Serialize};

use crate::sbor::{Decode, Encode};

use crate::rust::boxed::Box;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::*;

// For enum, we use internally tagged representation for readability.
// See: https://serde.rs/enum-representations.html

/// Represents a SBOR value.
#[cfg_attr(
    any(feature = "serde_std", feature = "serde_alloc"),
    derive(Serialize, Deserialize),
    serde(tag = "type")
)]
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode)]
pub enum Value {
    Unit,
    Bool { value: bool },
    I8 { value: i8 },
    I16 { value: i16 },
    I32 { value: i32 },
    I64 { value: i64 },
    I128 { value: String }, // i128/u128 is not supported by serde_json
    U8 { value: u8 },
    U16 { value: u16 },
    U32 { value: u32 },
    U64 { value: u64 },
    U128 { value: String },
    String { value: String },

    Option { value: Option<Box<Value>> },

    Box { value: Box<Value> },

    Array { elements: Vec<Value> },

    Tuple { elements: Vec<Value> },

    Struct { fields: Fields },

    Enum { index: u8, fields: Fields },

    Vec { elements: Vec<Value> },

    TreeSet { elements: Vec<Value> },

    TreeMap { elements: Vec<(Value, Value)> },

    HashSet { elements: Vec<Value> },

    HashMap { elements: Vec<(Value, Value)> },

    Custom { ty: u8, data: Vec<u8> },
}

#[cfg_attr(
    any(feature = "serde_std", feature = "serde_alloc"),
    derive(Serialize, Deserialize),
    serde(tag = "type")
)]
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode)]
pub enum Fields {
    Named { named: Vec<Value> },

    Unnamed { unnamed: Vec<Value> },

    Unit,
}

/// Parses any SBOR data.
pub fn parse_any(data: &[u8]) -> Result<Value, DecodeError> {
    let mut decoder = Decoder::with_type(data);
    let result = traverse(None, &mut decoder);
    decoder.check_end()?;
    result
}

fn traverse(ty_known: Option<u8>, dec: &mut Decoder) -> Result<Value, DecodeError> {
    let ty = match ty_known {
        Some(t) => t,
        None => dec.read_type()?,
    };

    match ty {
        constants::TYPE_UNIT => Ok(Value::Unit),
        constants::TYPE_BOOL => Ok(Value::Bool {
            value: <bool>::decode_value(dec)?,
        }),
        constants::TYPE_I8 => Ok(Value::I8 {
            value: <i8>::decode_value(dec)?,
        }),
        constants::TYPE_I16 => Ok(Value::I16 {
            value: <i16>::decode_value(dec)?,
        }),
        constants::TYPE_I32 => Ok(Value::I32 {
            value: <i32>::decode_value(dec)?,
        }),
        constants::TYPE_I64 => Ok(Value::I64 {
            value: <i64>::decode_value(dec)?,
        }),
        constants::TYPE_I128 => Ok(Value::I128 {
            value: <i128>::decode_value(dec)?.to_string(),
        }),
        constants::TYPE_U8 => Ok(Value::U8 {
            value: <u8>::decode_value(dec)?,
        }),
        constants::TYPE_U16 => Ok(Value::U16 {
            value: <u16>::decode_value(dec)?,
        }),
        constants::TYPE_U32 => Ok(Value::U32 {
            value: <u32>::decode_value(dec)?,
        }),
        constants::TYPE_U64 => Ok(Value::U64 {
            value: <u64>::decode_value(dec)?,
        }),
        constants::TYPE_U128 => Ok(Value::U128 {
            value: <u128>::decode_value(dec)?.to_string(),
        }),
        constants::TYPE_STRING => Ok(Value::String {
            value: <String>::decode_value(dec)?,
        }),
        constants::TYPE_OPTION => {
            // index
            let index = dec.read_u8()?;
            // optional value
            match index {
                0 => Ok(Value::Option { value: None }),
                1 => Ok(Value::Option {
                    value: Some(Box::new(traverse(None, dec)?)),
                }),
                _ => Err(DecodeError::InvalidIndex(index)),
            }
        }
        constants::TYPE_BOX => Ok(Value::Box {
            value: Box::new(traverse(None, dec)?),
        }),
        constants::TYPE_ARRAY => {
            // element type
            let ele_ty = dec.read_type()?;
            // length
            let len = dec.read_len()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(traverse(Some(ele_ty), dec)?);
            }
            Ok(Value::Array { elements })
        }
        constants::TYPE_TUPLE => {
            //length
            let len = dec.read_len()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(traverse(None, dec)?);
            }
            Ok(Value::Tuple { elements })
        }
        constants::TYPE_STRUCT => {
            // fields
            let fields = traverse_fields(dec)?;
            Ok(Value::Struct { fields })
        }
        constants::TYPE_ENUM => {
            // index
            let index = dec.read_u8()?;
            // fields
            let fields = traverse_fields(dec)?;
            Ok(Value::Enum { index, fields })
        }
        constants::TYPE_VEC => {
            // element type
            let ele_ty = dec.read_type()?;
            // length
            let len = dec.read_len()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(traverse(Some(ele_ty), dec)?);
            }
            Ok(Value::Vec { elements })
        }
        constants::TYPE_TREE_SET | constants::TYPE_HASH_SET => {
            // element type
            let ele_ty = dec.read_type()?;
            // length
            let len = dec.read_len()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(traverse(Some(ele_ty), dec)?);
            }
            if ty == constants::TYPE_TREE_SET {
                Ok(Value::TreeSet { elements })
            } else {
                Ok(Value::HashSet { elements })
            }
        }
        constants::TYPE_TREE_MAP | constants::TYPE_HASH_MAP => {
            // length
            let len = dec.read_len()?;
            // key type
            let key_ty = dec.read_type()?;
            // value type
            let value_ty = dec.read_type()?;
            // elements
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push((traverse(Some(key_ty), dec)?, traverse(Some(value_ty), dec)?));
            }
            if ty == constants::TYPE_TREE_MAP {
                Ok(Value::TreeMap { elements })
            } else {
                Ok(Value::HashMap { elements })
            }
        }
        // scrypto types
        _ => {
            if ty >= constants::CUSTOM_TYPE_START {
                // length
                let len = dec.read_len()?;
                let slice = dec.read_bytes(len)?;
                Ok(Value::Custom {
                    ty,
                    data: slice.to_vec(),
                })
            } else {
                Err(DecodeError::InvalidType {
                    expected: 0xff,
                    actual: ty,
                })
            }
        }
    }
}

fn traverse_fields(dec: &mut Decoder) -> Result<Fields, DecodeError> {
    let ty = dec.read_type()?;
    match ty {
        constants::TYPE_FIELDS_NAMED => {
            //length
            let len = dec.read_len()?;
            // named fields
            let mut named = Vec::new();
            for _ in 0..len {
                named.push(traverse(None, dec)?);
            }
            Ok(Fields::Named { named })
        }
        constants::TYPE_FIELDS_UNNAMED => {
            //length
            let len = dec.read_len()?;
            // named fields
            let mut unnamed = Vec::new();
            for _ in 0..len {
                unnamed.push(traverse(None, dec)?);
            }
            Ok(Fields::Unnamed { unnamed })
        }
        constants::TYPE_FIELDS_UNIT => Ok(Fields::Unit),
        _ => Err(DecodeError::InvalidType {
            expected: 0xff,
            actual: ty,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::boxed::Box;
    use crate::rust::collections::*;
    use crate::rust::string::String;
    use crate::rust::vec;
    use crate::rust::vec::Vec;

    #[derive(Encode)]
    struct TestStruct {
        x: u32,
    }

    #[derive(Encode)]
    enum TestEnum {
        A { x: u32 },
        B(u32),
        C,
    }

    #[derive(Encode)]
    struct TestData {
        a: (),
        b: bool,
        c: i8,
        d: i16,
        e: i32,
        f: i64,
        g: i128,
        h: u8,
        i: u16,
        j: u32,
        k: u64,
        l: u128,
        m: String,
        n: Option<u32>,
        o: Box<u32>,
        p: [u32; 3],
        q: (u32, u32),
        r: TestStruct,
        s: TestEnum,
        t: TestEnum,
        u: TestEnum,
        v: Vec<u32>,
        w: BTreeSet<u32>,
        x: HashSet<u32>,
        y: BTreeMap<u32, u32>,
        z: HashMap<u32, u32>,
    }

    #[test]
    pub fn test_parse_normal() {
        let mut set1 = BTreeSet::new();
        set1.insert(1);
        let mut set2 = HashSet::new();
        set2.insert(2);
        let mut map1 = BTreeMap::new();
        map1.insert(1, 2);
        let mut map2 = HashMap::new();
        map2.insert(1, 2);

        let data = TestData {
            a: (),
            b: true,
            c: 1,
            d: 2,
            e: 3,
            f: 4,
            g: 5,
            h: 6,
            i: 7,
            j: 8,
            k: 9,
            l: 10,
            m: String::from("abc"),
            n: Some(1),
            o: Box::new(1),
            p: [1, 2, 3],
            q: (1, 2),
            r: TestStruct { x: 1 },
            s: TestEnum::A { x: 1 },
            t: TestEnum::B(2),
            u: TestEnum::C,
            v: vec![1, 2],
            w: set1,
            x: set2,
            y: map1,
            z: map2,
        };
        let bytes = encode_with_type(Vec::new(), &data);
        let value = parse_any(&bytes).unwrap();

        assert_eq!(
            Value::Struct {
                fields: Fields::Named {
                    named: vec![
                        Value::Unit,
                        Value::Bool { value: true },
                        Value::I8 { value: 1 },
                        Value::I16 { value: 2 },
                        Value::I32 { value: 3 },
                        Value::I64 { value: 4 },
                        Value::I128 {
                            value: 5u128.to_string()
                        },
                        Value::U8 { value: 6 },
                        Value::U16 { value: 7 },
                        Value::U32 { value: 8 },
                        Value::U64 { value: 9 },
                        Value::U128 {
                            value: 10u128.to_string()
                        },
                        Value::String {
                            value: String::from("abc")
                        },
                        Value::Option {
                            value: Some(Box::new(Value::U32 { value: 1 }))
                        },
                        Value::Box {
                            value: Box::new(Value::U32 { value: 1 })
                        },
                        Value::Array {
                            elements: vec![
                                Value::U32 { value: 1 },
                                Value::U32 { value: 2 },
                                Value::U32 { value: 3 },
                            ]
                        },
                        Value::Tuple {
                            elements: vec![Value::U32 { value: 1 }, Value::U32 { value: 2 },]
                        },
                        Value::Struct {
                            fields: Fields::Named {
                                named: vec![Value::U32 { value: 1 }]
                            }
                        },
                        Value::Enum {
                            index: 0,
                            fields: Fields::Named {
                                named: vec![Value::U32 { value: 1 }]
                            }
                        },
                        Value::Enum {
                            index: 1,
                            fields: Fields::Unnamed {
                                unnamed: vec![Value::U32 { value: 2 }]
                            }
                        },
                        Value::Enum {
                            index: 2,
                            fields: Fields::Unit
                        },
                        Value::Vec {
                            elements: vec![Value::U32 { value: 1 }, Value::U32 { value: 2 },]
                        },
                        Value::TreeSet {
                            elements: vec![Value::U32 { value: 1 }]
                        },
                        Value::HashSet {
                            elements: vec![Value::U32 { value: 2 }]
                        },
                        Value::TreeMap {
                            elements: vec![(Value::U32 { value: 1 }, Value::U32 { value: 2 }),]
                        },
                        Value::HashMap {
                            elements: vec![(Value::U32 { value: 1 }, Value::U32 { value: 2 }),]
                        }
                    ]
                }
            },
            value
        );
    }

    #[test]
    pub fn test_parse_custom() {
        let bytes: Vec<u8> = vec![0x80, 0x02, 0x00, 0x00, 0x00, 0x01, 0x02];
        let value = parse_any(&bytes).unwrap();

        assert_eq!(
            Value::Custom {
                ty: 0x80,
                data: vec![1, 2]
            },
            value
        );
    }
}
