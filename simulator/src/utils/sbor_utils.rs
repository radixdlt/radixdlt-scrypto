use sbor::parse::*;
use sbor::*;
use scrypto::constants::*;
use scrypto::rust::borrow::Borrow;
use scrypto::rust::convert::TryFrom;
use scrypto::types::*;

pub fn format_sbor(data: &[u8]) -> Result<(String, Vec<BID>), DecodeError> {
    let value = parse_any(data)?;
    let mut acc = Vec::new();
    Ok((traverse(&value, &mut acc)?, acc))
}

fn traverse(value: &Value, acc: &mut Vec<BID>) -> Result<String, DecodeError> {
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
        Value::String(v) => Ok(v.to_string()),
        // rust types
        Value::Option(v) => match v {
            Some(x) => Ok(format!("Some({})", traverse(x.borrow(), acc)?)),
            None => Ok(String::from("None")),
        },
        Value::Box(v) => Ok(format!("Box({})", traverse(v.borrow(), acc)?)),
        Value::Array(elements) => traverse_vec(elements.iter(), "[", "]", acc),
        Value::Tuple(elements) => traverse_vec(elements.iter(), "(", ")", acc),
        Value::Struct(fields) => Ok(format!("Struct {}", traverse_fields(fields, acc)?)),
        Value::Enum(index, fields) => {
            Ok(format!("Enum::{} {}", index, traverse_fields(fields, acc)?))
        }
        // collections
        Value::Vec(elements) => traverse_vec(elements.iter(), "Vec { ", " }", acc),
        Value::TreeSet(elements) => traverse_vec(elements.iter(), "TreeSet { ", " }", acc),
        Value::HashSet(elements) => traverse_vec(elements.iter(), "HashSet { ", " }", acc),
        Value::TreeMap(elements) => traverse_map(elements.iter(), "TreeMap { ", " }", acc),
        Value::HashMap(elements) => traverse_map(elements.iter(), "HashMap { ", " }", acc),
        Value::Custom(ty, data) => match *ty {
            SCRYPTO_TYPE_U256 => Ok(<U256>::from_little_endian(data).to_string()),
            SCRYPTO_TYPE_ADDRESS => traverse_scrypto::<Address>("Address", ty, data),
            SCRYPTO_TYPE_H256 => traverse_scrypto::<H256>("H256", ty, data),
            SCRYPTO_TYPE_MID => traverse_scrypto::<MID>("MID", ty, data),
            SCRYPTO_TYPE_BID => traverse_scrypto::<BID>("BID", ty, data).and_then(|s| {
                acc.push(BID::try_from(data.as_slice()).unwrap());
                Ok(s)
            }),
            SCRYPTO_TYPE_RID => traverse_scrypto::<RID>("RID", ty, data),
            _ => Err(DecodeError::InvalidType {
                expected: 0xff,
                actual: *ty,
            }),
        },
    }
}

fn traverse_fields(fields: &Fields, acc: &mut Vec<BID>) -> Result<String, DecodeError> {
    match fields {
        Fields::Named(named) => traverse_vec(named.iter(), "{ ", " }", acc),
        Fields::Unnamed(unnamed) => traverse_vec(unnamed.iter(), "( ", " )", acc),
        Fields::Unit => Ok(String::from("()")),
    }
}

fn traverse_vec<'a, I: Iterator<Item = &'a Value>>(
    itr: I,
    begin: &str,
    end: &str,
    acc: &mut Vec<BID>,
) -> Result<String, DecodeError> {
    let mut buf = String::from(begin);
    for (i, x) in itr.enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(traverse(x, acc)?.as_str());
    }
    buf.push_str(end);
    Ok(buf)
}

fn traverse_map<'a, I: Iterator<Item = &'a (Value, Value)>>(
    itr: I,
    begin: &str,
    end: &str,
    acc: &mut Vec<BID>,
) -> Result<String, DecodeError> {
    let mut buf = String::from(begin);
    for (i, x) in itr.enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(format!("{}: {}", traverse(&x.0, acc)?, traverse(&x.1, acc)?).as_str());
    }
    buf.push_str(end);
    Ok(buf)
}

fn traverse_scrypto<'a, T: TryFrom<&'a [u8]> + ToString>(
    name: &str,
    ty: &u8,
    slice: &'a [u8],
) -> Result<String, DecodeError> {
    Ok(format!(
        "{}({})",
        name,
        <T>::try_from(slice)
            .map_err(|_| DecodeError::InvalidCustomData(*ty))?
            .to_string()
    ))
}
