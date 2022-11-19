use crate::decode::*;
use crate::decoder::*;
use crate::encode::*;
use crate::path::SborPathBuf;
use crate::rust::fmt::Debug;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::type_id::*;

/// CV is CustomValue
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SborValue<X: CustomTypeId, CV> {
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
        fields: Vec<SborValue<X, CV>>,
    },
    Enum {
        discriminator: String,
        fields: Vec<SborValue<X, CV>>,
    },
    Array {
        element_type_id: SborTypeId<X>,
        elements: Vec<SborValue<X, CV>>,
    },
    Tuple {
        elements: Vec<SborValue<X, CV>>,
    },
    Custom {
        value: CV,
    },
}

/// Encodes any SBOR value into byte array.
pub fn encode_any<X: CustomTypeId, CV: Encode<X>>(value: &SborValue<X, CV>) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_any_with_buffer(value, &mut bytes);
    bytes
}

/// Encodes any SBOR value with a given buffer
pub fn encode_any_with_buffer<X: CustomTypeId, CV: Encode<X>>(
    value: &SborValue<X, CV>,
    buffer: &mut Vec<u8>,
) {
    let mut encoder = ::sbor::Encoder::new(buffer);
    encode_type_id(value, &mut encoder);
    encode_body(value, &mut encoder);
}

fn encode_type_id<X: CustomTypeId, CV: Encode<X>>(
    value: &SborValue<X, CV>,
    encoder: &mut Encoder<X>,
) {
    match value {
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

fn encode_body<X: CustomTypeId, CV: Encode<X>>(value: &SborValue<X, CV>, encoder: &mut Encoder<X>) {
    match value {
        SborValue::Unit => {
            ().encode_body(encoder);
        }
        SborValue::Bool { value } => {
            value.encode_body(encoder);
        }
        SborValue::I8 { value } => {
            value.encode_body(encoder);
        }
        SborValue::I16 { value } => {
            value.encode_body(encoder);
        }
        SborValue::I32 { value } => {
            value.encode_body(encoder);
        }
        SborValue::I64 { value } => {
            value.encode_body(encoder);
        }
        SborValue::I128 { value } => {
            value.encode_body(encoder);
        }
        SborValue::U8 { value } => {
            value.encode_body(encoder);
        }
        SborValue::U16 { value } => {
            value.encode_body(encoder);
        }
        SborValue::U32 { value } => {
            value.encode_body(encoder);
        }
        SborValue::U64 { value } => {
            value.encode_body(encoder);
        }
        SborValue::U128 { value } => {
            value.encode_body(encoder);
        }
        SborValue::String { value } => {
            value.encode_body(encoder);
        }
        SborValue::Struct { fields } => {
            encoder.write_size(fields.len());
            for field in fields {
                encode_type_id(field, encoder);
                encode_body(field, encoder);
            }
        }
        SborValue::Enum {
            discriminator,
            fields,
        } => {
            encoder.write_discriminator(discriminator);
            encoder.write_size(fields.len());
            for field in fields {
                encode_type_id(field, encoder);
                encode_body(field, encoder);
            }
        }
        SborValue::Array {
            element_type_id,
            elements,
        } => {
            encoder.write_type_id(element_type_id.clone());
            encoder.write_size(elements.len());
            for e in elements {
                encode_body(e, encoder);
            }
        }
        SborValue::Tuple { elements } => {
            encoder.write_size(elements.len());
            for e in elements {
                encode_type_id(e, encoder);
                encode_body(e, encoder);
            }
        }
        // custom
        SborValue::Custom { value } => {
            value.encode_body(encoder);
        }
    }
}

/// Decode any SBOR data.
pub fn decode_any<X: CustomTypeId, CV: for<'a> Decode<X, VecDecoder<'a, X>>>(
    data: &[u8],
) -> Result<SborValue<X, CV>, DecodeError> {
    let mut decoder = VecDecoder::new(data);
    let type_id = decoder.read_type_id()?;
    let result = decode_body_with_type_id(type_id, &mut decoder)?;
    decoder.check_end()?;
    Ok(result)
}

fn decode_body_with_type_id<X: CustomTypeId, CV: Decode<X, D>, D: Decoder<X>>(
    type_id: SborTypeId<X>,
    decoder: &mut D,
) -> Result<SborValue<X, CV>, DecodeError> {
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
            let mut fields = Vec::new();
            for _ in 0..len {
                let type_id = decoder.read_type_id()?;
                fields.push(decode_body_with_type_id(type_id, decoder)?);
            }
            Ok(SborValue::Struct { fields })
        }
        SborTypeId::Enum => {
            // discriminator
            let discriminator = <String>::decode_body_with_type_id(decoder, String::type_id())?;
            // number of fields
            let len = decoder.read_size()?;
            // fields
            let mut fields = Vec::new();
            for _ in 0..len {
                let type_id = decoder.read_type_id()?;
                fields.push(decode_body_with_type_id(type_id, decoder)?);
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
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(decode_body_with_type_id(element_type_id, decoder)?);
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
            let mut elements = Vec::new();
            for _ in 0..len {
                let type_id = decoder.read_type_id()?;
                elements.push(decode_body_with_type_id(type_id, decoder)?);
            }
            Ok(SborValue::Tuple { elements })
        }
        SborTypeId::Custom(_) => Ok(SborValue::Custom {
            value: decoder.decode_body_with_type_id(type_id)?,
        }),
    }
}

pub fn traverse_any<X: CustomTypeId, CV, V: CustomValueVisitor<CV, Err = E>, E>(
    path: &mut SborPathBuf,
    value: &SborValue<X, CV>,
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

pub trait CustomValueVisitor<CV> {
    type Err;

    fn visit(&mut self, path: &mut SborPathBuf, value: &CV) -> Result<(), Self::Err>;
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
        let bytes = encode::<NoCustomTypeId, _>(&data);
        let value = decode_any::<NoCustomTypeId, NoCustomValue>(&bytes).unwrap();

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
            value
        );

        let mut bytes2 = Vec::new();
        let mut enc = Encoder::new(&mut bytes2);
        encode_type_id(&value, &mut enc);
        encode_body(&value, &mut enc);
        assert_eq!(bytes2, bytes);
    }
}
