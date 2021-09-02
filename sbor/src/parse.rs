use crate::rust::boxed::Box;
use crate::rust::string::String;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    String(String),

    Option(Option<Box<Value>>),

    Box(Box<Value>),

    Array(u8, Vec<Value>),

    Tuple(Vec<Value>),

    Struct(Fields),

    Enum(u8, Fields),

    Vec(u8, Vec<Value>),

    TreeSet(u8, Vec<Value>),

    TreeMap(u8, u8, Vec<(Value, Value)>),

    HashSet(u8, Vec<Value>),

    HashMap(u8, u8, Vec<(Value, Value)>),

    Custom(u8, Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fields {
    Named(Vec<Value>),

    Unnamed(Vec<Value>),

    Unit,
}

pub fn write_any(ty_ctx: Option<u8>, value: &Value, enc: &mut Encoder) {
    match value {
        // basic types
        Value::Unit => write_basic(ty_ctx, constants::TYPE_UNIT, &(), enc),
        Value::Bool(v) => write_basic(ty_ctx, constants::TYPE_BOOL, v, enc),
        Value::I8(v) => write_basic(ty_ctx, constants::TYPE_I8, v, enc),
        Value::I16(v) => write_basic(ty_ctx, constants::TYPE_I16, v, enc),
        Value::I32(v) => write_basic(ty_ctx, constants::TYPE_I32, v, enc),
        Value::I64(v) => write_basic(ty_ctx, constants::TYPE_I64, v, enc),
        Value::I128(v) => write_basic(ty_ctx, constants::TYPE_I128, v, enc),
        Value::U8(v) => write_basic(ty_ctx, constants::TYPE_U8, v, enc),
        Value::U16(v) => write_basic(ty_ctx, constants::TYPE_U16, v, enc),
        Value::U32(v) => write_basic(ty_ctx, constants::TYPE_U32, v, enc),
        Value::U64(v) => write_basic(ty_ctx, constants::TYPE_U64, v, enc),
        Value::U128(v) => write_basic(ty_ctx, constants::TYPE_U128, v, enc),
        Value::String(v) => write_basic(ty_ctx, constants::TYPE_STRING, v, enc),
        // rust types
        Value::Option(v) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_OPTION);
            }
            match v {
                Some(x) => {
                    enc.write_u8(1);
                    write_any(None, x, enc);
                }
                None => {
                    enc.write_u8(0);
                }
            }
        }
        Value::Box(v) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_BOX);
            }
            write_any(None, v, enc);
        }
        Value::Array(ty, elements) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_ARRAY);
            }
            enc.write_type(*ty);
            enc.write_len(elements.len());
            for e in elements {
                write_any(Some(*ty), e, enc);
            }
        }
        Value::Tuple(elements) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_TUPLE);
            }
            enc.write_len(elements.len());
            for e in elements {
                write_any(None, e, enc);
            }
        }
        Value::Struct(fields) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_STRUCT);
            }
            write_fields(fields, enc);
        }
        Value::Enum(index, fields) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_ENUM);
            }
            enc.write_u8(*index);
            write_fields(fields, enc);
        }
        // collections
        Value::Vec(ty, elements) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_VEC);
            }
            enc.write_type(*ty);
            enc.write_len(elements.len());
            for e in elements {
                write_any(Some(*ty), e, enc);
            }
        }
        Value::TreeSet(ty, elements) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_TREE_SET);
            }
            enc.write_type(*ty);
            enc.write_len(elements.len());
            for e in elements {
                write_any(Some(*ty), e, enc);
            }
        }
        Value::HashSet(ty, elements) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_HASH_SET);
            }
            enc.write_type(*ty);
            enc.write_len(elements.len());
            for e in elements {
                write_any(Some(*ty), e, enc);
            }
        }
        Value::TreeMap(ty_k, ty_v, elements) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_TREE_MAP);
            }
            enc.write_type(*ty_k);
            enc.write_type(*ty_v);
            enc.write_len(elements.len());
            for (k, v) in elements {
                write_any(Some(*ty_k), k, enc);
                write_any(Some(*ty_v), v, enc);
            }
        }
        Value::HashMap(ty_k, ty_v, elements) => {
            if ty_ctx.is_none() {
                enc.write_type(constants::TYPE_HASH_MAP);
            }
            enc.write_type(*ty_k);
            enc.write_type(*ty_v);
            enc.write_len(elements.len());
            for (k, v) in elements {
                write_any(Some(*ty_k), k, enc);
                write_any(Some(*ty_v), v, enc);
            }
        }
        // custom types
        Value::Custom(ty, data) => {
            if ty_ctx.is_none() {
                enc.write_type(*ty);
            }
            enc.write_len(data.len());
            enc.write_slice(data);
        }
    }
}

pub fn write_fields(fields: &Fields, enc: &mut Encoder) {
    match fields {
        Fields::Named(named) => {
            enc.write_type(constants::TYPE_FIELDS_NAMED);
            enc.write_len(named.len());
            for e in named {
                write_any(None, e, enc);
            }
        }
        Fields::Unnamed(unnamed) => {
            enc.write_type(constants::TYPE_FIELDS_UNNAMED);
            enc.write_len(unnamed.len());
            for e in unnamed {
                write_any(None, e, enc);
            }
        }
        Fields::Unit => {
            enc.write_type(constants::TYPE_FIELDS_UNIT);
        }
    }
}

fn write_basic<T: Encode>(ty_ctx: Option<u8>, t: u8, v: &T, enc: &mut Encoder) {
    if ty_ctx.is_none() {
        enc.write_type(t);
    }
    <T>::encode_value(v, enc);
}

/// Parses any SBOR data.
pub fn parse_any(data: &[u8]) -> Result<Value, DecodeError> {
    let mut decoder = Decoder::with_type(data);
    let result = traverse(None, &mut decoder);
    decoder.check_end()?;
    result
}

fn traverse(ty_ctx: Option<u8>, dec: &mut Decoder) -> Result<Value, DecodeError> {
    let ty = match ty_ctx {
        Some(t) => t,
        None => dec.read_type()?,
    };

    match ty {
        constants::TYPE_UNIT => Ok(Value::Unit),
        constants::TYPE_BOOL => Ok(Value::Bool(<bool>::decode_value(dec)?)),
        constants::TYPE_I8 => Ok(Value::I8(<i8>::decode_value(dec)?)),
        constants::TYPE_I16 => Ok(Value::I16(<i16>::decode_value(dec)?)),
        constants::TYPE_I32 => Ok(Value::I32(<i32>::decode_value(dec)?)),
        constants::TYPE_I64 => Ok(Value::I64(<i64>::decode_value(dec)?)),
        constants::TYPE_I128 => Ok(Value::I128(<i128>::decode_value(dec)?)),
        constants::TYPE_U8 => Ok(Value::U8(<u8>::decode_value(dec)?)),
        constants::TYPE_U16 => Ok(Value::U16(<u16>::decode_value(dec)?)),
        constants::TYPE_U32 => Ok(Value::U32(<u32>::decode_value(dec)?)),
        constants::TYPE_U64 => Ok(Value::U64(<u64>::decode_value(dec)?)),
        constants::TYPE_U128 => Ok(Value::U128(<u128>::decode_value(dec)?)),
        constants::TYPE_STRING => Ok(Value::String(<String>::decode_value(dec)?)),
        constants::TYPE_OPTION => {
            // index
            let index = dec.read_u8()?;
            // optional value
            match index {
                0 => Ok(Value::Option(None)),
                1 => Ok(Value::Option(Some(Box::new(traverse(None, dec)?)))),
                _ => Err(DecodeError::InvalidIndex(index)),
            }
        }
        constants::TYPE_BOX => Ok(Value::Box(Box::new(traverse(None, dec)?))),
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
            Ok(Value::Array(ele_ty, elements))
        }
        constants::TYPE_TUPLE => {
            //length
            let len = dec.read_len()?;
            // values
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push(traverse(None, dec)?);
            }
            Ok(Value::Tuple(elements))
        }
        constants::TYPE_STRUCT => {
            // fields
            let fields = traverse_fields(dec)?;
            Ok(Value::Struct(fields))
        }
        constants::TYPE_ENUM => {
            // index
            let index = dec.read_u8()?;
            // fields
            let fields = traverse_fields(dec)?;
            Ok(Value::Enum(index, fields))
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
            Ok(Value::Vec(ele_ty, elements))
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
                Ok(Value::TreeSet(ele_ty, elements))
            } else {
                Ok(Value::HashSet(ele_ty, elements))
            }
        }
        constants::TYPE_TREE_MAP | constants::TYPE_HASH_MAP => {
            // key type
            let key_ty = dec.read_type()?;
            // value type
            let value_ty = dec.read_type()?;
            // length
            let len = dec.read_len()?;
            // elements
            let mut elements = Vec::new();
            for _ in 0..len {
                elements.push((traverse(Some(key_ty), dec)?, traverse(Some(value_ty), dec)?));
            }
            if ty == constants::TYPE_TREE_MAP {
                Ok(Value::TreeMap(key_ty, value_ty, elements))
            } else {
                Ok(Value::HashMap(key_ty, value_ty, elements))
            }
        }
        _ => {
            if ty >= constants::CUSTOM_TYPE_START {
                // length
                let len = dec.read_len()?;
                let slice = dec.read_bytes(len)?;
                Ok(Value::Custom(ty, slice.to_vec()))
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
            Ok(Fields::Named(named))
        }
        constants::TYPE_FIELDS_UNNAMED => {
            //length
            let len = dec.read_len()?;
            // named fields
            let mut unnamed = Vec::new();
            for _ in 0..len {
                unnamed.push(traverse(None, dec)?);
            }
            Ok(Fields::Unnamed(unnamed))
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
            Value::Struct(Fields::Named(vec![
                Value::Unit,
                Value::Bool(true),
                Value::I8(1),
                Value::I16(2),
                Value::I32(3),
                Value::I64(4),
                Value::I128(5),
                Value::U8(6),
                Value::U16(7),
                Value::U32(8),
                Value::U64(9),
                Value::U128(10),
                Value::String(String::from("abc")),
                Value::Option(Some(Box::new(Value::U32(1)))),
                Value::Box(Box::new(Value::U32(1))),
                Value::Array(
                    constants::TYPE_U32,
                    vec![Value::U32(1), Value::U32(2), Value::U32(3),]
                ),
                Value::Tuple(vec![Value::U32(1), Value::U32(2),]),
                Value::Struct(Fields::Named(vec![Value::U32(1)])),
                Value::Enum(0, Fields::Named(vec![Value::U32(1)])),
                Value::Enum(1, Fields::Unnamed(vec![Value::U32(2)])),
                Value::Enum(2, Fields::Unit),
                Value::Vec(constants::TYPE_U32, vec![Value::U32(1), Value::U32(2),]),
                Value::TreeSet(constants::TYPE_U32, vec![Value::U32(1)]),
                Value::HashSet(constants::TYPE_U32, vec![Value::U32(2)]),
                Value::TreeMap(
                    constants::TYPE_U32,
                    constants::TYPE_U32,
                    vec![(Value::U32(1), Value::U32(2)),]
                ),
                Value::HashMap(
                    constants::TYPE_U32,
                    constants::TYPE_U32,
                    vec![(Value::U32(1), Value::U32(2)),]
                )
            ])),
            value
        );

        let mut enc = Encoder::with_type(Vec::new());
        write_any(None, &value, &mut enc);
        let bytes2: Vec<u8> = enc.into();
        assert_eq!(bytes2, bytes);
    }

    #[test]
    pub fn test_parse_custom() {
        let bytes: Vec<u8> = vec![0x80, 0x02, 0x00, 0x00, 0x00, 0x01, 0x02];
        let value = parse_any(&bytes).unwrap();

        assert_eq!(Value::Custom(0x80, vec![1, 2]), value);
    }
}
