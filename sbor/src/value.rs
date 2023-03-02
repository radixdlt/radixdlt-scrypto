use crate::decode::*;
use crate::decoder::*;
use crate::encode::*;
use crate::encoder::*;
use crate::path::SborPathBuf;
use crate::rust::fmt::Debug;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::value_kind::*;

/// Y is the CustomValue type. This is likely an enum, capturing all the custom values for the
/// particular SBOR extension.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<X: CustomValueKind, Y> {
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
    Enum {
        discriminator: u8,
        fields: Vec<Value<X, Y>>,
    },
    Array {
        element_value_kind: ValueKind<X>,
        elements: Vec<Value<X, Y>>,
    },
    Tuple {
        fields: Vec<Value<X, Y>>,
    },
    Map {
        key_value_kind: ValueKind<X>,
        value_value_kind: ValueKind<X>,
        entries: Vec<(Value<X, Y>, Value<X, Y>)>,
    },
    Custom {
        value: Y,
    },
}

impl<X: CustomValueKind, E: Encoder<X>, Y: Encode<X, E>> Encode<X, E> for Value<X, Y> {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Value::Bool { .. } => encoder.write_value_kind(ValueKind::Bool),
            Value::I8 { .. } => encoder.write_value_kind(ValueKind::I8),
            Value::I16 { .. } => encoder.write_value_kind(ValueKind::I16),
            Value::I32 { .. } => encoder.write_value_kind(ValueKind::I32),
            Value::I64 { .. } => encoder.write_value_kind(ValueKind::I64),
            Value::I128 { .. } => encoder.write_value_kind(ValueKind::I128),
            Value::U8 { .. } => encoder.write_value_kind(ValueKind::U8),
            Value::U16 { .. } => encoder.write_value_kind(ValueKind::U16),
            Value::U32 { .. } => encoder.write_value_kind(ValueKind::U32),
            Value::U64 { .. } => encoder.write_value_kind(ValueKind::U64),
            Value::U128 { .. } => encoder.write_value_kind(ValueKind::U128),
            Value::String { .. } => encoder.write_value_kind(ValueKind::String),
            Value::Enum { .. } => encoder.write_value_kind(ValueKind::Enum),
            Value::Array { .. } => encoder.write_value_kind(ValueKind::Array),
            Value::Tuple { .. } => encoder.write_value_kind(ValueKind::Tuple),
            Value::Map { .. } => encoder.write_value_kind(ValueKind::Map),
            Value::Custom { value } => value.encode_value_kind(encoder),
        }
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Value::Bool { value } => {
                value.encode_body(encoder)?;
            }
            Value::I8 { value } => {
                value.encode_body(encoder)?;
            }
            Value::I16 { value } => {
                value.encode_body(encoder)?;
            }
            Value::I32 { value } => {
                value.encode_body(encoder)?;
            }
            Value::I64 { value } => {
                value.encode_body(encoder)?;
            }
            Value::I128 { value } => {
                value.encode_body(encoder)?;
            }
            Value::U8 { value } => {
                value.encode_body(encoder)?;
            }
            Value::U16 { value } => {
                value.encode_body(encoder)?;
            }
            Value::U32 { value } => {
                value.encode_body(encoder)?;
            }
            Value::U64 { value } => {
                value.encode_body(encoder)?;
            }
            Value::U128 { value } => {
                value.encode_body(encoder)?;
            }
            Value::String { value } => {
                value.encode_body(encoder)?;
            }
            Value::Enum {
                discriminator,
                fields,
            } => {
                encoder.write_discriminator(*discriminator)?;
                encoder.write_size(fields.len())?;
                for field in fields {
                    encoder.encode(field)?;
                }
            }
            Value::Array {
                element_value_kind,
                elements,
            } => {
                encoder.write_value_kind(*element_value_kind)?;
                encoder.write_size(elements.len())?;
                for item in elements {
                    encoder.encode_deeper_body(item)?;
                }
            }
            Value::Tuple { fields } => {
                encoder.write_size(fields.len())?;
                for field in fields {
                    encoder.encode(field)?;
                }
            }
            Value::Map {
                key_value_kind,
                value_value_kind,
                entries,
            } => {
                encoder.write_value_kind(*key_value_kind)?;
                encoder.write_value_kind(*value_value_kind)?;
                encoder.write_size(entries.len())?;
                for entry in entries {
                    encoder.encode_deeper_body(&entry.0)?;
                    encoder.encode_deeper_body(&entry.1)?;
                }
            }
            // custom
            Value::Custom { value } => {
                value.encode_body(encoder)?;
            }
        }
        Ok(())
    }
}

impl<X: CustomValueKind, D: Decoder<X>, Y: Decode<X, D>> Decode<X, D> for Value<X, Y> {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        match value_kind {
            // primitive types
            ValueKind::Bool => Ok(Value::Bool {
                value: <bool>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::I8 => Ok(Value::I8 {
                value: <i8>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::I16 => Ok(Value::I16 {
                value: <i16>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::I32 => Ok(Value::I32 {
                value: <i32>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::I64 => Ok(Value::I64 {
                value: <i64>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::I128 => Ok(Value::I128 {
                value: <i128>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::U8 => Ok(Value::U8 {
                value: <u8>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::U16 => Ok(Value::U16 {
                value: <u16>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::U32 => Ok(Value::U32 {
                value: <u32>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::U64 => Ok(Value::U64 {
                value: <u64>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::U128 => Ok(Value::U128 {
                value: <u128>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::String => Ok(Value::String {
                value: <String>::decode_body_with_value_kind(decoder, value_kind)?,
            }),
            ValueKind::Tuple => {
                let length = decoder.read_size()?;
                let mut fields = Vec::with_capacity(if length <= 1024 { length } else { 1024 });
                for _ in 0..length {
                    fields.push(decoder.decode()?);
                }
                Ok(Value::Tuple { fields })
            }
            ValueKind::Enum => {
                let discriminator = decoder.read_discriminator()?;
                let length = decoder.read_size()?;
                let mut fields = Vec::with_capacity(if length <= 1024 { length } else { 1024 });
                for _ in 0..length {
                    fields.push(decoder.decode()?);
                }
                Ok(Value::Enum {
                    discriminator,
                    fields,
                })
            }
            ValueKind::Array => {
                let element_value_kind = decoder.read_value_kind()?;
                let length = decoder.read_size()?;
                let mut elements = Vec::with_capacity(if length <= 1024 { length } else { 1024 });
                for _ in 0..length {
                    elements.push(decoder.decode_deeper_body_with_value_kind(element_value_kind)?);
                }
                Ok(Value::Array {
                    element_value_kind,
                    elements,
                })
            }
            ValueKind::Map => {
                let key_value_kind = decoder.read_value_kind()?;
                let value_value_kind = decoder.read_value_kind()?;
                let length = decoder.read_size()?;
                let mut entries = Vec::with_capacity(if length <= 1024 { length } else { 1024 });
                for _ in 0..length {
                    entries.push((
                        decoder.decode_deeper_body_with_value_kind(key_value_kind)?,
                        decoder.decode_deeper_body_with_value_kind(value_value_kind)?,
                    ));
                }
                Ok(Value::Map {
                    key_value_kind,
                    value_value_kind,
                    entries,
                })
            }
            ValueKind::Custom(_) => Ok(Value::Custom {
                value: Y::decode_body_with_value_kind(decoder, value_kind)?,
            }),
        }
    }
}

pub use schema::*;

mod schema {
    use super::*;
    use crate::*;

    impl<X: CustomValueKind, Y, C: CustomTypeKind<GlobalTypeId>> Describe<C> for Value<X, Y> {
        const TYPE_ID: GlobalTypeId = GlobalTypeId::well_known(basic_well_known_types::ANY_ID);
    }
}

pub fn traverse_any<X: CustomValueKind, Y, V: ValueVisitor<X, Y, Err = E>, E>(
    path: &mut SborPathBuf,
    value: &Value<X, Y>,
    visitor: &mut V,
) -> Result<(), E> {
    match value {
        // primitive types
        Value::Bool { .. }
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
        Value::Tuple { fields } => {
            for (i, e) in fields.iter().enumerate() {
                path.push(i);
                traverse_any(path, e, visitor)?;
                path.pop();
            }
        }
        Value::Enum { fields, .. } => {
            for (i, field) in fields.iter().enumerate() {
                path.push(i);
                traverse_any(path, field, visitor)?;
                path.pop();
            }
        }
        Value::Array {
            element_value_kind,
            elements,
        } => {
            visitor.visit_array(path, element_value_kind, elements)?;
            for (i, e) in elements.iter().enumerate() {
                path.push(i);
                traverse_any(path, e, visitor)?;
                path.pop();
            }
        }
        Value::Map {
            key_value_kind,
            value_value_kind,
            entries,
        } => {
            visitor.visit_map(path, key_value_kind, value_value_kind, entries)?;
            for (i, e) in entries.iter().enumerate() {
                path.push(i);

                path.push(0);
                traverse_any(path, &e.0, visitor)?;
                path.pop();

                path.push(1);
                traverse_any(path, &e.1, visitor)?;
                path.pop();

                path.pop();
            }
        }
        // custom types
        Value::Custom { value } => {
            visitor.visit(path, value)?;
        }
    }

    Ok(())
}

pub trait ValueVisitor<X: CustomValueKind, Y> {
    type Err;

    fn visit_array(
        &mut self,
        _path: &mut SborPathBuf,
        _element_value_kind: &ValueKind<X>,
        _elements: &[Value<X, Y>],
    ) -> Result<(), Self::Err> {
        Ok(())
    }

    fn visit_map(
        &mut self,
        _path: &mut SborPathBuf,
        _key_value_kind: &ValueKind<X>,
        _value_value_kind: &ValueKind<X>,
        _entries: &[(Value<X, Y>, Value<X, Y>)],
    ) -> Result<(), Self::Err> {
        Ok(())
    }

    fn visit(&mut self, path: &mut SborPathBuf, value: &Y) -> Result<(), Self::Err>;
}

#[cfg(test)]
mod tests {
    use crate::rust::collections::*;
    use crate::rust::string::String;
    use crate::rust::vec;
    use crate::rust::vec::Vec;
    use crate::*;

    use super::*;

    #[derive(Categorize, Encode)]
    struct TestStruct {
        x: u32,
    }

    #[derive(Categorize, Encode)]
    enum TestEnum {
        A { x: u32 },
        B(u32),
        C,
    }

    #[derive(Categorize, Encode)]
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
        o: Result<u32, u32>,
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
        map2.insert(3, 4);

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
            o: Ok(2),
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
        let encoded_typed_value = basic_encode(&data).unwrap();
        let sbor_value = basic_decode(&encoded_typed_value).unwrap();

        assert_eq!(
            BasicValue::Tuple {
                fields: vec![
                    BasicValue::Tuple { fields: vec![] },
                    BasicValue::Bool { value: true },
                    BasicValue::I8 { value: 1 },
                    BasicValue::I16 { value: 2 },
                    BasicValue::I32 { value: 3 },
                    BasicValue::I64 { value: 4 },
                    BasicValue::I128 { value: 5 },
                    BasicValue::U8 { value: 6 },
                    BasicValue::U16 { value: 7 },
                    BasicValue::U32 { value: 8 },
                    BasicValue::U64 { value: 9 },
                    BasicValue::U128 { value: 10 },
                    BasicValue::String {
                        value: String::from("abc")
                    },
                    BasicValue::Enum {
                        discriminator: 1,
                        fields: vec![BasicValue::U32 { value: 1 }]
                    },
                    BasicValue::Enum {
                        discriminator: 0,
                        fields: vec![BasicValue::U32 { value: 2 }]
                    },
                    BasicValue::Array {
                        element_value_kind: ValueKind::U32,
                        elements: vec![
                            BasicValue::U32 { value: 1 },
                            BasicValue::U32 { value: 2 },
                            BasicValue::U32 { value: 3 },
                        ]
                    },
                    BasicValue::Tuple {
                        fields: vec![BasicValue::U32 { value: 1 }, BasicValue::U32 { value: 2 },]
                    },
                    BasicValue::Tuple {
                        fields: vec![BasicValue::U32 { value: 1 }]
                    },
                    BasicValue::Enum {
                        discriminator: 0,
                        fields: vec![BasicValue::U32 { value: 1 }]
                    },
                    BasicValue::Enum {
                        discriminator: 1,
                        fields: vec![BasicValue::U32 { value: 2 }]
                    },
                    BasicValue::Enum {
                        discriminator: 2,
                        fields: vec![]
                    },
                    BasicValue::Array {
                        element_value_kind: ValueKind::U32,
                        elements: vec![BasicValue::U32 { value: 1 }, BasicValue::U32 { value: 2 },]
                    },
                    BasicValue::Array {
                        element_value_kind: ValueKind::U32,
                        elements: vec![BasicValue::U32 { value: 1 }]
                    },
                    BasicValue::Array {
                        element_value_kind: ValueKind::U32,
                        elements: vec![BasicValue::U32 { value: 2 }]
                    },
                    BasicValue::Map {
                        key_value_kind: ValueKind::U32,
                        value_value_kind: ValueKind::U32,
                        entries: vec![(BasicValue::U32 { value: 1 }, BasicValue::U32 { value: 2 })]
                    },
                    BasicValue::Map {
                        key_value_kind: ValueKind::U32,
                        value_value_kind: ValueKind::U32,
                        entries: vec![(BasicValue::U32 { value: 3 }, BasicValue::U32 { value: 4 })]
                    }
                ]
            },
            sbor_value
        );

        let encoded_sbor_value = basic_encode(&sbor_value).unwrap();

        assert_eq!(encoded_sbor_value, encoded_typed_value);
    }

    #[test]
    pub fn test_max_depth_array_decode_behaviour() {
        let allowable_payload = encode_array_of_depth(BASIC_SBOR_V1_MAX_DEPTH).unwrap();
        let allowable_result = basic_decode::<BasicValue>(&allowable_payload);
        assert!(allowable_result.is_ok());

        let forbidden_payload = encode_array_of_depth(BASIC_SBOR_V1_MAX_DEPTH + 1).unwrap();
        let forbidden_result = basic_decode::<BasicValue>(&forbidden_payload);
        assert!(forbidden_result.is_err());
    }

    #[test]
    pub fn test_max_depth_struct_decode_behaviour() {
        let allowable_payload = encode_struct_of_depth(BASIC_SBOR_V1_MAX_DEPTH).unwrap();
        let allowable_result = basic_decode::<BasicValue>(&allowable_payload);
        assert!(allowable_result.is_ok());

        let forbidden_payload = encode_struct_of_depth(BASIC_SBOR_V1_MAX_DEPTH + 1).unwrap();
        let forbidden_result = basic_decode::<BasicValue>(&forbidden_payload);
        assert!(forbidden_result.is_err());
    }

    #[test]
    pub fn test_max_depth_tuple_decode_behaviour() {
        let allowable_payload = encode_tuple_of_depth(BASIC_SBOR_V1_MAX_DEPTH).unwrap();
        let allowable_result = basic_decode::<BasicValue>(&allowable_payload);
        assert!(allowable_result.is_ok());

        let forbidden_payload = encode_tuple_of_depth(BASIC_SBOR_V1_MAX_DEPTH + 1).unwrap();
        let forbidden_result = basic_decode::<BasicValue>(&forbidden_payload);
        assert!(forbidden_result.is_err());
    }

    pub fn encode_array_of_depth(depth: usize) -> Result<Vec<u8>, EncodeError> {
        let mut buf = Vec::new();
        let mut encoder = BasicEncoder::new(&mut buf, 256);
        encoder.write_payload_prefix(BASIC_SBOR_V1_PAYLOAD_PREFIX)?;
        encoder.write_value_kind(ValueKind::Array)?;
        // Encodes depth - 1 array bodies
        for _ in 1..depth {
            encoder.write_value_kind(ValueKind::Array)?; // Child type
            encoder.write_size(1)?;
        }
        // And finishes off encoding a single layer
        encoder.write_value_kind(ValueKind::Array)?; // Child type
        encoder.write_size(0)?;

        Ok(buf)
    }

    pub fn encode_struct_of_depth(depth: usize) -> Result<Vec<u8>, EncodeError> {
        let mut buf = Vec::new();
        let mut encoder = BasicEncoder::new(&mut buf, 256);
        encoder.write_payload_prefix(BASIC_SBOR_V1_PAYLOAD_PREFIX)?;
        // Encodes depth - 1 structs containing 1 child
        for _ in 1..depth {
            encoder.write_value_kind(ValueKind::Tuple)?;
            encoder.write_size(1)?;
        }
        // And finishes off encoding a single layer with 0 children
        encoder.write_value_kind(ValueKind::Tuple)?;
        encoder.write_size(0)?;

        Ok(buf)
    }

    pub fn encode_tuple_of_depth(depth: usize) -> Result<Vec<u8>, EncodeError> {
        let mut buf = Vec::new();
        let mut encoder = BasicEncoder::new(&mut buf, 256);
        encoder.write_payload_prefix(BASIC_SBOR_V1_PAYLOAD_PREFIX)?;
        // Encodes depth - 1 structs containing 1 child
        for _ in 1..depth {
            encoder.write_value_kind(ValueKind::Tuple)?;
            encoder.write_size(1)?;
        }
        // And finishes off encoding a single layer with 0 children
        encoder.write_value_kind(ValueKind::Tuple)?;
        encoder.write_size(0)?;

        Ok(buf)
    }
}
