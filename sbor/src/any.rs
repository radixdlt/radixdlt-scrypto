use sbor::path::MutableSborPath;

use crate::decode::*;
use crate::encode::*;
use crate::rust::borrow::Borrow;
use crate::rust::boxed::Box;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::type_id::*;

/// Represents a SBOR value.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // For JSON readability, see https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool {
        value: bool,
    },
    I8 {
        value: i8,
    },
    I16 {
        value: i16,
    },
    I32 {
        value: i32,
    },
    I64 {
        value: i64,
    },
    I128 {
        value: i128,
    },
    U8 {
        value: u8,
    },
    U16 {
        value: u16,
    },
    U32 {
        value: u32,
    },
    U64 {
        value: u64,
    },
    U128 {
        value: u128,
    },
    String {
        value: String,
    },

    Struct {
        fields: Vec<Value>,
    },
    Enum {
        name: String,
        fields: Vec<Value>,
    },

    Option {
        value: Box<Option<Value>>,
    },
    Array {
        element_type_id: u8,
        elements: Vec<Value>,
    },
    Tuple {
        elements: Vec<Value>,
    },
    Result {
        value: Box<Result<Value, Value>>,
    },

    Vec {
        element_type_id: u8,
        elements: Vec<Value>,
    },
    TreeSet {
        element_type_id: u8,
        elements: Vec<Value>,
    },
    TreeMap {
        key_type_id: u8,
        value_type_id: u8,
        elements: Vec<Value>,
    },
    HashSet {
        element_type_id: u8,
        elements: Vec<Value>,
    },
    HashMap {
        key_type_id: u8,
        value_type_id: u8,
        elements: Vec<Value>,
    },
    Custom {
        type_id: u8,
        #[cfg_attr(feature = "serde", serde(with = "hex::serde"))]
        bytes: Vec<u8>,
    },
}

/// Encodes any SBOR value into byte array.
pub fn encode_any(value: &Value) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut enc = ::sbor::Encoder::with_type(&mut bytes);
    encode_any_internal(None, value, &mut enc);
    bytes
}

/// Encodes any SBOR value with a given buffer
pub fn encode_any_with_buffer(value: &Value, buffer: &mut Vec<u8>) {
    let mut enc = ::sbor::Encoder::with_type(buffer);
    encode_any_internal(None, value, &mut enc);
}

fn encode_any_internal(ty_ctx: Option<u8>, value: &Value, enc: &mut Encoder) {
    match value {
        // primitive types
        Value::Unit => encode_basic(ty_ctx, TYPE_UNIT, &(), enc),
        Value::Bool { value } => encode_basic(ty_ctx, TYPE_BOOL, value, enc),
        Value::I8 { value } => encode_basic(ty_ctx, TYPE_I8, value, enc),
        Value::I16 { value } => encode_basic(ty_ctx, TYPE_I16, value, enc),
        Value::I32 { value } => encode_basic(ty_ctx, TYPE_I32, value, enc),
        Value::I64 { value } => encode_basic(ty_ctx, TYPE_I64, value, enc),
        Value::I128 { value } => encode_basic(ty_ctx, TYPE_I128, value, enc),
        Value::U8 { value } => encode_basic(ty_ctx, TYPE_U8, value, enc),
        Value::U16 { value } => encode_basic(ty_ctx, TYPE_U16, value, enc),
        Value::U32 { value } => encode_basic(ty_ctx, TYPE_U32, value, enc),
        Value::U64 { value } => encode_basic(ty_ctx, TYPE_U64, value, enc),
        Value::U128 { value } => encode_basic(ty_ctx, TYPE_U128, value, enc),
        Value::String { value } => encode_basic(ty_ctx, TYPE_STRING, value, enc),
        // struct & enum
        Value::Struct { fields } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_STRUCT);
            }
            enc.write_len(fields.len());
            for field in fields {
                encode_any_internal(None, field, enc);
            }
        }
        Value::Enum { name, fields } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_ENUM);
            }
            name.encode_value(enc);
            enc.write_len(fields.len());
            for field in fields {
                encode_any_internal(None, field, enc);
            }
        }
        // composite types
        Value::Option { value } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_OPTION);
            }
            match value.borrow() {
                None => {
                    enc.write_u8(0);
                }
                Some(x) => {
                    enc.write_u8(1);
                    encode_any_internal(None, x, enc);
                }
            }
        }
        Value::Array {
            element_type_id,
            elements,
        } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_ARRAY);
            }
            enc.write_type(*element_type_id);
            enc.write_len(elements.len());
            for e in elements {
                encode_any_internal(Some(*element_type_id), e, enc);
            }
        }
        Value::Tuple { elements } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_TUPLE);
            }
            enc.write_len(elements.len());
            for e in elements {
                encode_any_internal(None, e, enc);
            }
        }
        Value::Result { value } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_RESULT);
            }
            match value.borrow() {
                Ok(x) => {
                    enc.write_u8(0);
                    encode_any_internal(None, x, enc);
                }
                Err(x) => {
                    enc.write_u8(1);
                    encode_any_internal(None, x, enc);
                }
            }
        }
        // collections
        Value::Vec {
            element_type_id,
            elements,
        } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_VEC);
            }
            enc.write_type(*element_type_id);
            enc.write_len(elements.len());
            for e in elements {
                encode_any_internal(Some(*element_type_id), e, enc);
            }
        }
        Value::TreeSet {
            element_type_id,
            elements,
        } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_TREE_SET);
            }
            enc.write_type(*element_type_id);
            enc.write_len(elements.len());
            for e in elements {
                encode_any_internal(Some(*element_type_id), e, enc);
            }
        }
        Value::HashSet {
            element_type_id,
            elements,
        } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_HASH_SET);
            }
            enc.write_type(*element_type_id);
            enc.write_len(elements.len());
            for e in elements {
                encode_any_internal(Some(*element_type_id), e, enc);
            }
        }
        Value::TreeMap {
            key_type_id,
            value_type_id,
            elements,
        } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_TREE_MAP);
            }
            enc.write_type(*key_type_id);
            enc.write_type(*value_type_id);
            enc.write_len(elements.len() / 2);
            for pair in elements.chunks(2) {
                encode_any_internal(Some(*key_type_id), &pair[0], enc);
                encode_any_internal(Some(*value_type_id), &pair[1], enc);
            }
        }
        Value::HashMap {
            key_type_id,
            value_type_id,
            elements,
        } => {
            if ty_ctx.is_none() {
                enc.write_type(TYPE_HASH_MAP);
            }
            enc.write_type(*key_type_id);
            enc.write_type(*value_type_id);
            enc.write_len(elements.len() / 2);
            for pair in elements.chunks(2) {
                encode_any_internal(Some(*key_type_id), &pair[0], enc);
                encode_any_internal(Some(*value_type_id), &pair[1], enc);
            }
        }
        // custom
        Value::Custom { type_id, bytes } => {
            if ty_ctx.is_none() {
                enc.write_type(*type_id);
            }
            enc.write_len(bytes.len());
            enc.write_slice(bytes);
        }
    }
}

fn encode_basic<T: Encode>(ty_ctx: Option<u8>, t: u8, v: &T, enc: &mut Encoder) {
    if ty_ctx.is_none() {
        enc.write_type(t);
    }
    <T>::encode_value(v, enc);
}

/// Decode any SBOR data.
pub fn decode_any(data: &[u8]) -> Result<Value, DecodeError> {
    let mut decoder = Decoder::with_type(data);
    let result = decode_next(None, &mut decoder);
    decoder.check_end()?;
    result
}

fn decode_next(ty_ctx: Option<u8>, dec: &mut Decoder) -> Result<Value, DecodeError> {
    let ty = match ty_ctx {
        Some(t) => t,
        None => dec.read_type()?,
    };

    match ty {
        // primitive types
        TYPE_UNIT => Ok(Value::Unit),
        TYPE_BOOL => Ok(Value::Bool {
            value: <bool>::decode_value(dec)?,
        }),
        TYPE_I8 => Ok(Value::I8 {
            value: <i8>::decode_value(dec)?,
        }),
        TYPE_I16 => Ok(Value::I16 {
            value: <i16>::decode_value(dec)?,
        }),
        TYPE_I32 => Ok(Value::I32 {
            value: <i32>::decode_value(dec)?,
        }),
        TYPE_I64 => Ok(Value::I64 {
            value: <i64>::decode_value(dec)?,
        }),
        TYPE_I128 => Ok(Value::I128 {
            value: <i128>::decode_value(dec)?,
        }),
        TYPE_U8 => Ok(Value::U8 {
            value: <u8>::decode_value(dec)?,
        }),
        TYPE_U16 => Ok(Value::U16 {
            value: <u16>::decode_value(dec)?,
        }),
        TYPE_U32 => Ok(Value::U32 {
            value: <u32>::decode_value(dec)?,
        }),
        TYPE_U64 => Ok(Value::U64 {
            value: <u64>::decode_value(dec)?,
        }),
        TYPE_U128 => Ok(Value::U128 {
            value: <u128>::decode_value(dec)?,
        }),
        TYPE_STRING => Ok(Value::String {
            value: <String>::decode_value(dec)?,
        }),
        // struct & enum
        TYPE_STRUCT => {
            // number of fields
            let len = dec.read_len()?;
            // fields
            let mut fields = Vec::new();
            for _ in 0..len {
                fields.push(decode_next(None, dec)?);
            }
            Ok(Value::Struct { fields })
        }
        TYPE_ENUM => {
            // name
            let name = <String>::decode_value(dec)?;
            // number of fields
            let len = dec.read_len()?;
            // fields
            let mut fields = Vec::new();
            for _ in 0..len {
                fields.push(decode_next(None, dec)?);
            }
            Ok(Value::Enum { name, fields })
        }
        // composite types
        TYPE_OPTION => {
            // index
            let index = dec.read_u8()?;
            // optional value
            match index {
                0 => Ok(Value::Option {
                    value: Box::new(None),
                }),
                1 => Ok(Value::Option {
                    value: Box::new(Some(decode_next(None, dec)?)),
                }),
                _ => Err(DecodeError::InvalidIndex(index)),
            }
        }
        TYPE_ARRAY => {
            // element type
            let element_type_id = dec.read_type()?;
            // length
            let len = dec.read_len()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(decode_next(Some(element_type_id), dec)?);
            }
            Ok(Value::Array {
                element_type_id,
                elements,
            })
        }
        TYPE_TUPLE => {
            //length
            let len = dec.read_len()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(decode_next(None, dec)?);
            }
            Ok(Value::Tuple { elements })
        }
        TYPE_RESULT => {
            // index
            let index = dec.read_u8()?;
            // result value
            match index {
                0 => Ok(Value::Result {
                    value: Box::new(Ok(decode_next(None, dec)?)),
                }),
                1 => Ok(Value::Result {
                    value: Box::new(Err(decode_next(None, dec)?)),
                }),
                _ => Err(DecodeError::InvalidIndex(index)),
            }
        }
        // collections
        TYPE_VEC => {
            // element type
            let element_type_id = dec.read_type()?;
            // length
            let len = dec.read_len()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(decode_next(Some(element_type_id), dec)?);
            }
            Ok(Value::Vec {
                element_type_id,
                elements,
            })
        }
        TYPE_TREE_SET | TYPE_HASH_SET => {
            // element type
            let element_type_id = dec.read_type()?;
            // length
            let len = dec.read_len()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(decode_next(Some(element_type_id), dec)?);
            }
            if ty == TYPE_TREE_SET {
                Ok(Value::TreeSet {
                    element_type_id,
                    elements,
                })
            } else {
                Ok(Value::HashSet {
                    element_type_id,
                    elements,
                })
            }
        }
        TYPE_TREE_MAP | TYPE_HASH_MAP => {
            // key type
            let key_type_id = dec.read_type()?;
            // value type
            let value_type_id = dec.read_type()?;
            // length
            let len = dec.read_len()?;
            // elements
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(decode_next(Some(key_type_id), dec)?);
                elements.push(decode_next(Some(value_type_id), dec)?);
            }
            if ty == TYPE_TREE_MAP {
                Ok(Value::TreeMap {
                    key_type_id,
                    value_type_id,
                    elements,
                })
            } else {
                Ok(Value::HashMap {
                    key_type_id,
                    value_type_id,
                    elements,
                })
            }
        }
        _ => {
            if ty >= TYPE_CUSTOM_START {
                // length
                let len = dec.read_len()?;
                let slice = dec.read_bytes(len)?;
                Ok(Value::Custom {
                    type_id: ty,
                    bytes: slice.to_vec(),
                })
            } else {
                Err(DecodeError::InvalidType {
                    expected: None,
                    actual: ty,
                })
            }
        }
    }
}

pub fn traverse_any<V, E>(
    path: &mut MutableSborPath,
    value: &Value,
    visitor: &mut V,
) -> Result<(), E>
where
    V: CustomValueVisitor<Err = E>,
{
    match value {
        // primitive types
        Value::Unit
        | Value::Bool { .. }
        | Value::I8 { .. }
        | Value::I16 { .. }
        | Value::I32 { .. }
        | Value::I64 { .. }
        | Value::I128 { .. }
        | Value::U8 { .. }
        | Value::U16 { .. }
        | Value::U32 { .. }
        | Value::U64 { .. }
        | Value::U128 { .. }
        | Value::String { .. } => {}
        // struct & enum
        Value::Struct { fields } | Value::Enum { fields, .. } => {
            for (i, field) in fields.iter().enumerate() {
                path.push(i);
                traverse_any(path, field, visitor)?;
                path.pop();
            }
        }
        // composite types
        Value::Option { value } => match value.borrow() {
            None => {}
            Some(x) => {
                path.push(0);
                traverse_any(path, x, visitor)?;
                path.pop();
            }
        },
        Value::Array { elements, .. } => {
            for (i, e) in elements.iter().enumerate() {
                path.push(i);
                traverse_any(path, e, visitor)?;
                path.pop();
            }
        }
        Value::Tuple { elements } => {
            for (i, e) in elements.iter().enumerate() {
                path.push(i);
                traverse_any(path, e, visitor)?;
                path.pop();
            }
        }
        Value::Result { value } => match value.borrow() {
            Ok(x) | Err(x) => {
                path.push(0);
                traverse_any(path, x, visitor)?;
                path.pop();
            }
        },
        // collections
        Value::Vec { elements, .. }
        | Value::TreeSet { elements, .. }
        | Value::HashSet { elements, .. }
        | Value::TreeMap { elements, .. }
        | Value::HashMap { elements, .. } => {
            for (i, e) in elements.iter().enumerate() {
                path.push(i);
                traverse_any(path, e, visitor)?;
                path.pop();
            }
        }
        // custom types
        Value::Custom { type_id, bytes } => {
            visitor.visit(path, *type_id, bytes)?;
        }
    }

    Ok(())
}

pub trait CustomValueVisitor {
    type Err;

    fn visit(
        &mut self,
        path: &mut MutableSborPath,
        type_id: u8,
        data: &[u8],
    ) -> Result<(), Self::Err>;
}

#[cfg(test)]
mod tests {
    use crate::rust::boxed::Box;
    use crate::rust::collections::*;
    use crate::rust::string::String;
    use crate::rust::string::ToString;
    use crate::rust::vec;
    use crate::rust::vec::Vec;
    use crate::*;

    use super::*;

    #[derive(TypeId, Encode)]
    struct TestStruct {
        x: u32,
    }

    #[derive(TypeId, Encode)]
    enum TestEnum {
        A { x: u32 },
        B(u32),
        C,
    }

    #[derive(TypeId, Encode)]
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
        let bytes = encode_with_type(&data);
        let value = decode_any(&bytes).unwrap();

        assert_eq!(
            Value::Struct {
                fields: vec![
                    Value::Unit,
                    Value::Bool { value: true },
                    Value::I8 { value: 1 },
                    Value::I16 { value: 2 },
                    Value::I32 { value: 3 },
                    Value::I64 { value: 4 },
                    Value::I128 { value: 5 },
                    Value::U8 { value: 6 },
                    Value::U16 { value: 7 },
                    Value::U32 { value: 8 },
                    Value::U64 { value: 9 },
                    Value::U128 { value: 10 },
                    Value::String {
                        value: String::from("abc")
                    },
                    Value::Option {
                        value: Box::new(Some(Value::U32 { value: 1 }))
                    },
                    Value::Array {
                        element_type_id: TYPE_U32,
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
                        fields: vec![Value::U32 { value: 1 }]
                    },
                    Value::Enum {
                        name: "A".to_string(),
                        fields: vec![Value::U32 { value: 1 }]
                    },
                    Value::Enum {
                        name: "B".to_string(),
                        fields: vec![Value::U32 { value: 2 }]
                    },
                    Value::Enum {
                        name: "C".to_string(),
                        fields: vec![]
                    },
                    Value::Vec {
                        element_type_id: TYPE_U32,
                        elements: vec![Value::U32 { value: 1 }, Value::U32 { value: 2 },]
                    },
                    Value::TreeSet {
                        element_type_id: TYPE_U32,
                        elements: vec![Value::U32 { value: 1 }]
                    },
                    Value::HashSet {
                        element_type_id: TYPE_U32,
                        elements: vec![Value::U32 { value: 2 }]
                    },
                    Value::TreeMap {
                        key_type_id: TYPE_U32,
                        value_type_id: TYPE_U32,
                        elements: vec![Value::U32 { value: 1 }, Value::U32 { value: 2 }]
                    },
                    Value::HashMap {
                        key_type_id: TYPE_U32,
                        value_type_id: TYPE_U32,
                        elements: vec![Value::U32 { value: 1 }, Value::U32 { value: 2 }]
                    }
                ]
            },
            value
        );

        let mut bytes2 = Vec::new();
        let mut enc = Encoder::with_type(&mut bytes2);
        encode_any_internal(None, &value, &mut enc);
        assert_eq!(bytes2, bytes);
    }

    #[test]
    pub fn test_parse_custom() {
        let bytes: Vec<u8> = vec![0x80, 0x02, 0x00, 0x00, 0x00, 0x01, 0x02];
        let value = decode_any(&bytes).unwrap();

        assert_eq!(
            Value::Custom {
                type_id: 0x80,
                bytes: vec![1, 2]
            },
            value
        );
    }
}
