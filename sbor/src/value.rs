use sbor::path::SborPathBuf;

use crate::decode::*;
use crate::encode::*;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::type_id::*;
use crate::*;

/// Represents a SBOR value.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // For JSON readability, see https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum SborValue {
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
        fields: Vec<SborValue>,
    },
    Enum {
        discriminator: String,
        fields: Vec<SborValue>,
    },
    Array {
        element_type_id: SborTypeId,
        elements: Vec<SborValue>,
    },
    Tuple {
        elements: Vec<SborValue>,
    },

    Custom {
        type_id: SborTypeId,
        #[cfg_attr(feature = "serde", serde(with = "hex::serde"))]
        bytes: Vec<u8>,
    },
}

/// Encodes any SBOR value into byte array.
pub fn encode_any(value: &SborValue) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut enc = ::sbor::Encoder::new(&mut bytes);
    encode_value(None, value, &mut enc);
    bytes
}

/// Encodes any SBOR value with a given buffer
pub fn encode_any_with_buffer(value: &SborValue, buffer: &mut Vec<u8>) {
    let mut enc = ::sbor::Encoder::new(buffer);
    encode_value(None, value, &mut enc);
}

fn encode_value(known_type: Option<SborTypeId>, value: &SborValue, enc: &mut Encoder) {
    match value {
        SborValue::Unit => encode_basic(known_type, SborTypeId::Unit, &(), enc),
        SborValue::Bool { value } => encode_basic(known_type, SborTypeId::Bool, value, enc),
        SborValue::I8 { value } => encode_basic(known_type, SborTypeId::I8, value, enc),
        SborValue::I16 { value } => encode_basic(known_type, SborTypeId::I16, value, enc),
        SborValue::I32 { value } => encode_basic(known_type, SborTypeId::I32, value, enc),
        SborValue::I64 { value } => encode_basic(known_type, SborTypeId::I64, value, enc),
        SborValue::I128 { value } => encode_basic(known_type, SborTypeId::I128, value, enc),
        SborValue::U8 { value } => encode_basic(known_type, SborTypeId::U8, value, enc),
        SborValue::U16 { value } => encode_basic(known_type, SborTypeId::U16, value, enc),
        SborValue::U32 { value } => encode_basic(known_type, SborTypeId::U32, value, enc),
        SborValue::U64 { value } => encode_basic(known_type, SborTypeId::U64, value, enc),
        SborValue::U128 { value } => encode_basic(known_type, SborTypeId::U128, value, enc),
        SborValue::String { value } => encode_basic(known_type, SborTypeId::String, value, enc),
        SborValue::Struct { fields } => {
            if known_type.is_none() {
                enc.write_type_id(SborTypeId::Struct);
            }
            enc.write_size(fields.len());
            for field in fields {
                encode_value(None, field, enc);
            }
        }
        SborValue::Enum {
            discriminator,
            fields,
        } => {
            if known_type.is_none() {
                enc.write_type_id(SborTypeId::Enum);
            }
            enc.write_discriminator(discriminator);
            enc.write_size(fields.len());
            for field in fields {
                encode_value(None, field, enc);
            }
        }
        SborValue::Array {
            element_type_id,
            elements,
        } => {
            if known_type.is_none() {
                enc.write_type_id(SborTypeId::Array);
            }
            enc.write_type_id(*element_type_id);
            enc.write_size(elements.len());
            for e in elements {
                encode_value(Some(*element_type_id), e, enc);
            }
        }
        SborValue::Tuple { elements } => {
            if known_type.is_none() {
                enc.write_type_id(SborTypeId::Tuple);
            }
            enc.write_size(elements.len());
            for e in elements {
                encode_value(None, e, enc);
            }
        }
        // custom
        SborValue::Custom { type_id, bytes } => {
            if known_type.is_none() {
                enc.write_type_id(*type_id);
            }
            enc.write_size(bytes.len());
            enc.write_slice(bytes);
        }
    }
}

fn encode_basic<T: Encode>(
    known_type: Option<SborTypeId>,
    t: SborTypeId,
    v: &T,
    enc: &mut Encoder,
) {
    if known_type.is_none() {
        enc.write_type_id(t);
    }
    <T>::encode_value(v, enc);
}

/// Decode any SBOR data.
pub fn decode_any(data: &[u8]) -> Result<SborValue, DecodeError> {
    let mut decoder = Decoder::new(data);
    let result = decode_next(None, &mut decoder);
    decoder.check_end()?;
    result
}

fn decode_next(
    known_type: Option<SborTypeId>,
    dec: &mut Decoder,
) -> Result<SborValue, DecodeError> {
    let ty = match known_type {
        Some(t) => t,
        None => dec.read_type_id()?,
    };

    match ty {
        // primitive types
        SborTypeId::Unit => {
            <()>::decode_value(dec)?;
            Ok(SborValue::Unit)
        }
        SborTypeId::Bool => Ok(SborValue::Bool {
            value: <bool>::decode_value(dec)?,
        }),
        SborTypeId::I8 => Ok(SborValue::I8 {
            value: <i8>::decode_value(dec)?,
        }),
        SborTypeId::I16 => Ok(SborValue::I16 {
            value: <i16>::decode_value(dec)?,
        }),
        SborTypeId::I32 => Ok(SborValue::I32 {
            value: <i32>::decode_value(dec)?,
        }),
        SborTypeId::I64 => Ok(SborValue::I64 {
            value: <i64>::decode_value(dec)?,
        }),
        SborTypeId::I128 => Ok(SborValue::I128 {
            value: <i128>::decode_value(dec)?,
        }),
        SborTypeId::U8 => Ok(SborValue::U8 {
            value: <u8>::decode_value(dec)?,
        }),
        SborTypeId::U16 => Ok(SborValue::U16 {
            value: <u16>::decode_value(dec)?,
        }),
        SborTypeId::U32 => Ok(SborValue::U32 {
            value: <u32>::decode_value(dec)?,
        }),
        SborTypeId::U64 => Ok(SborValue::U64 {
            value: <u64>::decode_value(dec)?,
        }),
        SborTypeId::U128 => Ok(SborValue::U128 {
            value: <u128>::decode_value(dec)?,
        }),
        SborTypeId::String => Ok(SborValue::String {
            value: <String>::decode_value(dec)?,
        }),
        // struct & enum
        SborTypeId::Struct => {
            // number of fields
            let len = dec.read_size()?;
            // fields
            let mut fields = Vec::new();
            for _ in 0..len {
                fields.push(decode_next(None, dec)?);
            }
            Ok(SborValue::Struct { fields })
        }
        SborTypeId::Enum => {
            // discriminator
            let discriminator = <String>::decode_value(dec)?;
            // number of fields
            let len = dec.read_size()?;
            // fields
            let mut fields = Vec::new();
            for _ in 0..len {
                fields.push(decode_next(None, dec)?);
            }
            Ok(SborValue::Enum {
                discriminator,
                fields,
            })
        }
        // composite types
        SborTypeId::Array => {
            // element type
            let element_type_id = dec.read_type_id()?;
            // length
            let len = dec.read_size()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(decode_next(Some(element_type_id), dec)?);
            }
            Ok(SborValue::Array {
                element_type_id,
                elements,
            })
        }
        SborTypeId::Tuple => {
            //length
            let len = dec.read_size()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(decode_next(None, dec)?);
            }
            Ok(SborValue::Tuple { elements })
        }
        type_id @ SborTypeId::Custom(_) => {
            // length
            let len = dec.read_size()?;
            let slice = dec.read_slice(len)?;
            Ok(SborValue::Custom {
                type_id,
                bytes: slice.to_vec(),
            })
        }
    }
}

pub fn traverse_any<V, E>(
    path: &mut SborPathBuf,
    value: &SborValue,
    visitor: &mut V,
) -> Result<(), E>
where
    V: CustomValueVisitor<Err = E>,
{
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
        SborValue::Custom { type_id, bytes } => {
            visitor.visit(path, *type_id, bytes)?;
        }
    }

    Ok(())
}

pub trait CustomValueVisitor {
    type Err;

    fn visit(
        &mut self,
        path: &mut SborPathBuf,
        type_id: SborTypeId,
        data: &[u8],
    ) -> Result<(), Self::Err>;
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
        let bytes = encode(&data);
        let value = decode_any(&bytes).unwrap();

        assert_eq!(
            SborValue::Struct {
                fields: vec![
                    SborValue::Unit,
                    SborValue::Bool { value: true },
                    SborValue::I8 { value: 1 },
                    SborValue::I16 { value: 2 },
                    SborValue::I32 { value: 3 },
                    SborValue::I64 { value: 4 },
                    SborValue::I128 { value: 5 },
                    SborValue::U8 { value: 6 },
                    SborValue::U16 { value: 7 },
                    SborValue::U32 { value: 8 },
                    SborValue::U64 { value: 9 },
                    SborValue::U128 { value: 10 },
                    SborValue::String {
                        value: String::from("abc")
                    },
                    SborValue::Enum {
                        discriminator: "Some".to_string(),
                        fields: vec![SborValue::U32 { value: 1 }]
                    },
                    SborValue::Enum {
                        discriminator: "Ok".to_string(),
                        fields: vec![SborValue::U32 { value: 2 }]
                    },
                    SborValue::Array {
                        element_type_id: SborTypeId::U32,
                        elements: vec![
                            SborValue::U32 { value: 1 },
                            SborValue::U32 { value: 2 },
                            SborValue::U32 { value: 3 },
                        ]
                    },
                    SborValue::Tuple {
                        elements: vec![SborValue::U32 { value: 1 }, SborValue::U32 { value: 2 },]
                    },
                    SborValue::Struct {
                        fields: vec![SborValue::U32 { value: 1 }]
                    },
                    SborValue::Enum {
                        discriminator: "A".to_string(),
                        fields: vec![SborValue::U32 { value: 1 }]
                    },
                    SborValue::Enum {
                        discriminator: "B".to_string(),
                        fields: vec![SborValue::U32 { value: 2 }]
                    },
                    SborValue::Enum {
                        discriminator: "C".to_string(),
                        fields: vec![]
                    },
                    SborValue::Array {
                        element_type_id: SborTypeId::U32,
                        elements: vec![SborValue::U32 { value: 1 }, SborValue::U32 { value: 2 },]
                    },
                    SborValue::Array {
                        element_type_id: SborTypeId::U32,
                        elements: vec![SborValue::U32 { value: 1 }]
                    },
                    SborValue::Array {
                        element_type_id: SborTypeId::U32,
                        elements: vec![SborValue::U32 { value: 2 }]
                    },
                    SborValue::Array {
                        element_type_id: SborTypeId::Tuple,
                        elements: vec![SborValue::Tuple {
                            elements: vec![
                                SborValue::U32 { value: 1 },
                                SborValue::U32 { value: 2 }
                            ]
                        }]
                    },
                    SborValue::Array {
                        element_type_id: SborTypeId::Tuple,
                        elements: vec![SborValue::Tuple {
                            elements: vec![
                                SborValue::U32 { value: 3 },
                                SborValue::U32 { value: 4 }
                            ]
                        }]
                    }
                ]
            },
            value
        );

        let mut bytes2 = Vec::new();
        let mut enc = Encoder::new(&mut bytes2);
        encode_value(None, &value, &mut enc);
        assert_eq!(bytes2, bytes);
    }

    #[test]
    pub fn test_parse_custom() {
        let bytes: Vec<u8> = vec![0x80, 0x02, 0x00, 0x00, 0x00, 0x01, 0x02];
        let value = decode_any(&bytes).unwrap();

        assert_eq!(
            SborValue::Custom {
                type_id: SborTypeId::Custom(0x80),
                bytes: vec![1, 2]
            },
            value
        );
    }
}
