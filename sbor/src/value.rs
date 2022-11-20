use crate::decode::*;
use crate::decoder::*;
use crate::encode::*;
use crate::encoder::*;
use crate::path::SborPathBuf;
use crate::rust::fmt::Debug;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::type_id::*;

/// Y is the CustomValue type. This is likely an enum, capturing all the custom values for the
/// particular SBOR extension.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SborValue<X: CustomTypeId, Y> {
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
        fields: Vec<SborValue<X, Y>>,
    },
    Enum {
        discriminator: String,
        fields: Vec<SborValue<X, Y>>,
    },
    Array {
        element_type_id: SborTypeId<X>,
        elements: Vec<SborValue<X, Y>>,
    },
    Tuple {
        elements: Vec<SborValue<X, Y>>,
    },
    Custom {
        value: Y,
    },
}

impl<X: CustomTypeId, E: Encoder<X>, Y: Encode<X, E>> Encode<X, E> for SborValue<X, Y> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            SborValue::Unit => encoder.write_type_id(SborTypeId::Unit),
            SborValue::Bool { .. } => encoder.write_type_id(SborTypeId::Bool),
            SborValue::I8 { .. } => encoder.write_type_id(SborTypeId::I8),
            SborValue::I16 { .. } => encoder.write_type_id(SborTypeId::I16),
            SborValue::I32 { .. } => encoder.write_type_id(SborTypeId::I32),
            SborValue::I64 { .. } => encoder.write_type_id(SborTypeId::I64),
            SborValue::I128 { .. } => encoder.write_type_id(SborTypeId::I128),
            SborValue::U8 { .. } => encoder.write_type_id(SborTypeId::U8),
            SborValue::U16 { .. } => encoder.write_type_id(SborTypeId::U16),
            SborValue::U32 { .. } => encoder.write_type_id(SborTypeId::U32),
            SborValue::U64 { .. } => encoder.write_type_id(SborTypeId::U64),
            SborValue::U128 { .. } => encoder.write_type_id(SborTypeId::U128),
            SborValue::String { .. } => encoder.write_type_id(SborTypeId::String),
            SborValue::Struct { .. } => encoder.write_type_id(SborTypeId::Struct),
            SborValue::Enum { .. } => encoder.write_type_id(SborTypeId::Enum),
            SborValue::Array { .. } => encoder.write_type_id(SborTypeId::Array),
            SborValue::Tuple { .. } => encoder.write_type_id(SborTypeId::Tuple),
            SborValue::Custom { value } => value.encode_type_id(encoder),
        }
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            SborValue::Unit => {
                (()).encode_body(encoder)?;
            }
            SborValue::Bool { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::I8 { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::I16 { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::I32 { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::I64 { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::I128 { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::U8 { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::U16 { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::U32 { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::U64 { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::U128 { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::String { value } => {
                value.encode_body(encoder)?;
            }
            SborValue::Struct { fields } => {
                encoder.write_size(fields.len())?;
                for field in fields {
                    encoder.encode(field)?;
                }
            }
            SborValue::Enum {
                discriminator,
                fields,
            } => {
                encoder.write_discriminator(discriminator)?;
                encoder.write_size(fields.len())?;
                for field in fields {
                    encoder.encode(field)?;
                }
            }
            SborValue::Array {
                element_type_id,
                elements,
            } => {
                encoder.write_type_id(*element_type_id)?;
                encoder.write_size(elements.len())?;
                for item in elements {
                    encoder.encode_deeper_body(item)?;
                }
            }
            SborValue::Tuple { elements } => {
                encoder.write_size(elements.len())?;
                for field in elements {
                    encoder.encode(field)?;
                }
            }
            // custom
            SborValue::Custom { value } => {
                value.encode_body(encoder)?;
            }
        }
        Ok(())
    }
}

impl<X: CustomTypeId, D: Decoder<X>, Y: Decode<X, D>> Decode<X, D> for SborValue<X, Y> {
    #[inline]
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        match type_id {
            // primitive types
            SborTypeId::Unit => {
                <()>::decode_body_with_type_id(decoder, type_id)?;
                Ok(SborValue::Unit)
            }
            SborTypeId::Bool => Ok(SborValue::Bool {
                value: <bool>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::I8 => Ok(SborValue::I8 {
                value: <i8>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::I16 => Ok(SborValue::I16 {
                value: <i16>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::I32 => Ok(SborValue::I32 {
                value: <i32>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::I64 => Ok(SborValue::I64 {
                value: <i64>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::I128 => Ok(SborValue::I128 {
                value: <i128>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::U8 => Ok(SborValue::U8 {
                value: <u8>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::U16 => Ok(SborValue::U16 {
                value: <u16>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::U32 => Ok(SborValue::U32 {
                value: <u32>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::U64 => Ok(SborValue::U64 {
                value: <u64>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::U128 => Ok(SborValue::U128 {
                value: <u128>::decode_body_with_type_id(decoder, type_id)?,
            }),
            SborTypeId::String => Ok(SborValue::String {
                value: <String>::decode_body_with_type_id(decoder, type_id)?,
            }),
            // struct & enum
            SborTypeId::Struct => {
                // number of fields
                let len = decoder.read_size()?;
                // fields
                let mut fields = Vec::with_capacity(if len <= 1024 { len } else { 1024 });
                for _ in 0..len {
                    fields.push(decoder.decode()?);
                }
                Ok(SborValue::Struct { fields })
            }
            SborTypeId::Enum => {
                // discriminator
                let discriminator = decoder.read_discriminator()?;
                // number of fields
                let len = decoder.read_size()?;
                // fields
                let mut fields = Vec::with_capacity(if len <= 1024 { len } else { 1024 });
                for _ in 0..len {
                    fields.push(decoder.decode()?);
                }
                Ok(SborValue::Enum {
                    discriminator,
                    fields,
                })
            }
            // composite types
            SborTypeId::Array => {
                // element type
                let element_type_id = decoder.read_type_id()?;
                // length
                let len = decoder.read_size()?;
                // values
                let mut elements = Vec::with_capacity(if len <= 1024 { len } else { 1024 });
                for _ in 0..len {
                    elements.push(decoder.decode_deeper_body_with_type_id(element_type_id)?);
                }
                Ok(SborValue::Array {
                    element_type_id,
                    elements,
                })
            }
            SborTypeId::Tuple => {
                //length
                let len = decoder.read_size()?;
                // values
                let mut elements = Vec::with_capacity(if len <= 1024 { len } else { 1024 });
                for _ in 0..len {
                    elements.push(decoder.decode()?);
                }
                Ok(SborValue::Tuple { elements })
            }
            SborTypeId::Custom(_) => Ok(SborValue::Custom {
                value: Y::decode_body_with_type_id(decoder, type_id)?,
            }),
        }
    }
}

pub fn traverse_any<X: CustomTypeId, Y, V: CustomValueVisitor<Y, Err = E>, E>(
    path: &mut SborPathBuf,
    value: &SborValue<X, Y>,
    visitor: &mut V,
) -> Result<(), E> {
    match value {
        // primitive types
        SborValue::Unit
        | SborValue::Bool { .. }
        | SborValue::I8 { .. }
        | SborValue::I16 { .. }
        | SborValue::I32 { .. }
        | SborValue::I64 { .. }
        | SborValue::I128 { .. }
        | SborValue::U8 { .. }
        | SborValue::U16 { .. }
        | SborValue::U32 { .. }
        | SborValue::U64 { .. }
        | SborValue::U128 { .. }
        | SborValue::String { .. } => {}
        // struct & enum
        SborValue::Struct { fields } | SborValue::Enum { fields, .. } => {
            for (i, field) in fields.iter().enumerate() {
                path.push(i);
                traverse_any(path, field, visitor)?;
                path.pop();
            }
        }
        // composite types
        SborValue::Array { elements, .. } => {
            for (i, e) in elements.iter().enumerate() {
                path.push(i);
                traverse_any(path, e, visitor)?;
                path.pop();
            }
        }
        SborValue::Tuple { elements } => {
            for (i, e) in elements.iter().enumerate() {
                path.push(i);
                traverse_any(path, e, visitor)?;
                path.pop();
            }
        }
        // custom types
        SborValue::Custom { value } => {
            visitor.visit(path, value)?;
        }
    }

    Ok(())
}

pub trait CustomValueVisitor<Y> {
    type Err;

    fn visit(&mut self, path: &mut SborPathBuf, value: &Y) -> Result<(), Self::Err>;
}

#[cfg(test)]
mod tests {
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
            BasicSborValue::Struct {
                fields: vec![
                    BasicSborValue::Unit,
                    BasicSborValue::Bool { value: true },
                    BasicSborValue::I8 { value: 1 },
                    BasicSborValue::I16 { value: 2 },
                    BasicSborValue::I32 { value: 3 },
                    BasicSborValue::I64 { value: 4 },
                    BasicSborValue::I128 { value: 5 },
                    BasicSborValue::U8 { value: 6 },
                    BasicSborValue::U16 { value: 7 },
                    BasicSborValue::U32 { value: 8 },
                    BasicSborValue::U64 { value: 9 },
                    BasicSborValue::U128 { value: 10 },
                    BasicSborValue::String {
                        value: String::from("abc")
                    },
                    BasicSborValue::Enum {
                        discriminator: "Some".to_string(),
                        fields: vec![BasicSborValue::U32 { value: 1 }]
                    },
                    BasicSborValue::Enum {
                        discriminator: "Ok".to_string(),
                        fields: vec![BasicSborValue::U32 { value: 2 }]
                    },
                    BasicSborValue::Array {
                        element_type_id: SborTypeId::U32,
                        elements: vec![
                            BasicSborValue::U32 { value: 1 },
                            BasicSborValue::U32 { value: 2 },
                            BasicSborValue::U32 { value: 3 },
                        ]
                    },
                    BasicSborValue::Tuple {
                        elements: vec![
                            BasicSborValue::U32 { value: 1 },
                            BasicSborValue::U32 { value: 2 },
                        ]
                    },
                    BasicSborValue::Struct {
                        fields: vec![BasicSborValue::U32 { value: 1 }]
                    },
                    BasicSborValue::Enum {
                        discriminator: "A".to_string(),
                        fields: vec![BasicSborValue::U32 { value: 1 }]
                    },
                    BasicSborValue::Enum {
                        discriminator: "B".to_string(),
                        fields: vec![BasicSborValue::U32 { value: 2 }]
                    },
                    BasicSborValue::Enum {
                        discriminator: "C".to_string(),
                        fields: vec![]
                    },
                    BasicSborValue::Array {
                        element_type_id: SborTypeId::U32,
                        elements: vec![
                            BasicSborValue::U32 { value: 1 },
                            BasicSborValue::U32 { value: 2 },
                        ]
                    },
                    BasicSborValue::Array {
                        element_type_id: SborTypeId::U32,
                        elements: vec![BasicSborValue::U32 { value: 1 }]
                    },
                    BasicSborValue::Array {
                        element_type_id: SborTypeId::U32,
                        elements: vec![BasicSborValue::U32 { value: 2 }]
                    },
                    BasicSborValue::Array {
                        element_type_id: SborTypeId::Tuple,
                        elements: vec![BasicSborValue::Tuple {
                            elements: vec![
                                BasicSborValue::U32 { value: 1 },
                                BasicSborValue::U32 { value: 2 }
                            ]
                        }]
                    },
                    BasicSborValue::Array {
                        element_type_id: SborTypeId::Tuple,
                        elements: vec![BasicSborValue::Tuple {
                            elements: vec![
                                BasicSborValue::U32 { value: 3 },
                                BasicSborValue::U32 { value: 4 }
                            ]
                        }]
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
        let allowable_payload = encode_array_of_depth(DEFAULT_BASIC_MAX_DEPTH).unwrap();
        let allowable_result = basic_decode::<BasicSborValue>(&allowable_payload);
        assert!(allowable_result.is_ok());

        let forbidden_payload = encode_array_of_depth(DEFAULT_BASIC_MAX_DEPTH + 1).unwrap();
        let forbidden_result = basic_decode::<BasicSborValue>(&forbidden_payload);
        assert!(forbidden_result.is_err());
    }

    #[test]
    pub fn test_max_depth_struct_decode_behaviour() {
        let allowable_payload = encode_struct_of_depth(DEFAULT_BASIC_MAX_DEPTH).unwrap();
        let allowable_result = basic_decode::<BasicSborValue>(&allowable_payload);
        assert!(allowable_result.is_ok());

        let forbidden_payload = encode_struct_of_depth(DEFAULT_BASIC_MAX_DEPTH + 1).unwrap();
        let forbidden_result = basic_decode::<BasicSborValue>(&forbidden_payload);
        assert!(forbidden_result.is_err());
    }

    #[test]
    pub fn test_max_depth_tuple_decode_behaviour() {
        let allowable_payload = encode_tuple_of_depth(DEFAULT_BASIC_MAX_DEPTH).unwrap();
        let allowable_result = basic_decode::<BasicSborValue>(&allowable_payload);
        assert!(allowable_result.is_ok());

        let forbidden_payload = encode_tuple_of_depth(DEFAULT_BASIC_MAX_DEPTH + 1).unwrap();
        let forbidden_result = basic_decode::<BasicSborValue>(&forbidden_payload);
        assert!(forbidden_result.is_err());
    }

    pub fn encode_array_of_depth(depth: u8) -> Result<Vec<u8>, EncodeError> {
        let mut buf = Vec::new();
        let mut encoder = BasicEncoder::new(&mut buf);
        encoder.write_payload_prefix()?;
        encoder.write_type_id(SborTypeId::Array)?;
        // Encodes depth - 1 array bodies
        for _ in 1..depth {
            encoder.write_type_id(SborTypeId::Array)?; // Child type
            encoder.write_size(1)?;
        }
        // And finishes off encoding a single layer
        encoder.write_type_id(SborTypeId::Array)?; // Child type
        encoder.write_size(0)?;

        Ok(buf)
    }

    pub fn encode_struct_of_depth(depth: u8) -> Result<Vec<u8>, EncodeError> {
        let mut buf = Vec::new();
        let mut encoder = BasicEncoder::new(&mut buf);
        encoder.write_payload_prefix()?;
        // Encodes depth - 1 structs containing 1 child
        for _ in 1..depth {
            encoder.write_type_id(SborTypeId::Struct)?;
            encoder.write_size(1)?;
        }
        // And finishes off encoding a single layer with 0 children
        encoder.write_type_id(SborTypeId::Struct)?;
        encoder.write_size(0)?;

        Ok(buf)
    }

    pub fn encode_tuple_of_depth(depth: u8) -> Result<Vec<u8>, EncodeError> {
        let mut buf = Vec::new();
        let mut encoder = BasicEncoder::new(&mut buf);
        encoder.write_payload_prefix()?;
        // Encodes depth - 1 structs containing 1 child
        for _ in 1..depth {
            encoder.write_type_id(SborTypeId::Tuple)?;
            encoder.write_size(1)?;
        }
        // And finishes off encoding a single layer with 0 children
        encoder.write_type_id(SborTypeId::Tuple)?;
        encoder.write_size(0)?;

        Ok(buf)
    }
}
