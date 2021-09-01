use sbor::parse::*;
use sbor::*;
use scrypto::constants::*;
use scrypto::rust::borrow::Borrow;
use scrypto::rust::convert::TryFrom;
use scrypto::types::*;

pub fn format_sbor(data: &[u8]) -> Result<String, DecodeError> {
    let value = parse_any(data)?;
    traverse(&value)
}

fn traverse(value: &Value) -> Result<String, DecodeError> {
    match value {
        // basic types
        Value::Unit => Ok(String::from("()")),
        Value::Bool(v) => Ok(v.to_string()),
        Value::I8(v) => Ok(v.to_string()),
        Value::I16(v) => Ok(v.to_string()),
        Value::I32(v) => Ok(v.to_string()),
        Value::I64(v) => Ok(v.to_string()),
        Value::I128(v) => Ok(v.to_string()),
        Value::U8(v) => Ok(v.to_string()),
        Value::U16(v) => Ok(v.to_string()),
        Value::U32(v) => Ok(v.to_string()),
        Value::U64(v) => Ok(v.to_string()),
        Value::U128(v) => Ok(v.to_string()),
        Value::String(v) => Ok(v.clone()),
        // rust types
        Value::Option(v) => match v {
            Some(x) => Ok(format!("Some({})", traverse(x.borrow())?)),
            None => Ok(String::from("None")),
        },
        Value::Box(v) => Ok(format!("Box({})", traverse(v.borrow())?)),
        Value::Array(elements) => traverse_vec(elements.iter(), "[", "]"),
        Value::Tuple(elements) => traverse_vec(elements.iter(), "(", ")"),
        Value::Struct(fields) => Ok(format!("Struct {}", traverse_fields(fields)?)),
        Value::Enum(index, fields) => Ok(format!("Enum::{} {}", index, traverse_fields(fields)?)),
        // collections
        Value::Vec(elements) => traverse_vec(elements.iter(), "Vec {", "}"),
        Value::TreeSet(elements) => traverse_vec(elements.iter(), "TreeSet {", "}"),
        Value::HashSet(elements) => traverse_vec(elements.iter(), "HashSet {", "}"),
        Value::TreeMap(elements) => traverse_map(elements.iter(), "TreeMap {", "}"),
        Value::HashMap(elements) => traverse_map(elements.iter(), "HashMap {", "}"),
        Value::Custom(ty, data) => match *ty {
            SCRYPTO_TYPE_U256 => Ok(<U256>::from_little_endian(data).to_string()),
            SCRYPTO_TYPE_ADDRESS => Ok(<Address>::try_from(data.as_slice())
                .map_err(|_| DecodeError::InvalidCustomData(*ty))?
                .to_string()),
            SCRYPTO_TYPE_H256 => Ok(<H256>::try_from(data.as_slice())
                .map_err(|_| DecodeError::InvalidCustomData(*ty))?
                .to_string()),
            SCRYPTO_TYPE_MID => Ok(<MID>::try_from(data.as_slice())
                .map_err(|_| DecodeError::InvalidCustomData(*ty))?
                .to_string()),
            SCRYPTO_TYPE_BID => Ok(<BID>::try_from(data.as_slice())
                .map_err(|_| DecodeError::InvalidCustomData(*ty))?
                .to_string()),
            SCRYPTO_TYPE_RID => Ok(<RID>::try_from(data.as_slice())
                .map_err(|_| DecodeError::InvalidCustomData(*ty))?
                .to_string()),
            _ => Err(DecodeError::InvalidType {
                expected: 0xff,
                actual: *ty,
            }),
        },
    }
}

fn traverse_fields(fields: &Fields) -> Result<String, DecodeError> {
    match fields {
        Fields::Named(named) => traverse_vec(named.iter(), "{", "}"),
        Fields::Unnamed(unnamed) => traverse_vec(unnamed.iter(), "(", ")"),
        Fields::Unit => Ok(String::from("()")),
    }
}

fn traverse_vec<'a, I: Iterator<Item = &'a Value>>(
    itr: I,
    begin: &str,
    end: &str,
) -> Result<String, DecodeError> {
    let mut buf = String::from(begin);
    for (i, x) in itr.enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(traverse(x)?.as_str());
    }
    buf.push_str(end);
    Ok(buf)
}

fn traverse_map<'a, I: Iterator<Item = &'a (Value, Value)>>(
    itr: I,
    begin: &str,
    end: &str,
) -> Result<String, DecodeError> {
    let mut buf = String::from(begin);
    for (i, x) in itr.enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(format!("{}: {}", traverse(&x.0)?, traverse(&x.1)?).as_str());
    }
    buf.push_str(end);
    Ok(buf)
}
