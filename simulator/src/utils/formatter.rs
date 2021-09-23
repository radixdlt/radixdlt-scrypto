use sbor::parse::*;
use sbor::*;
use scrypto::constants::*;
use scrypto::rust::borrow::Borrow;
use scrypto::rust::convert::TryFrom;
use scrypto::types::*;

pub fn format_sbor(data: &[u8]) -> Result<String, DecodeError> {
    let value = parse_any(data)?;
    format_value(&value)
}

pub fn format_value(value: &Value) -> Result<String, DecodeError> {
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
        Value::Option(v) => match v.borrow() {
            Some(x) => Ok(format!("Some({})", format_value(x)?)),
            None => Ok(String::from("None")),
        },
        Value::Box(v) => Ok(format!("Box({})", format_value(v.borrow())?)),
        Value::Array(_, elements) => format_vec(elements.iter(), "[", "]"),
        Value::Tuple(elements) => format_vec(elements.iter(), "(", ")"),
        Value::Struct(fields) => Ok(format!("Struct {}", format_fields(fields)?)),
        Value::Enum(index, fields) => Ok(format!("Enum::{} {}", index, format_fields(fields)?)),
        // collections
        Value::Vec(_, elements) => format_vec(elements.iter(), "Vec { ", " }"),
        Value::TreeSet(_, elements) => format_vec(elements.iter(), "TreeSet { ", " }"),
        Value::HashSet(_, elements) => format_vec(elements.iter(), "HashSet { ", " }"),
        Value::TreeMap(_, _, elements) => format_map(elements.iter(), "TreeMap { ", " }"),
        Value::HashMap(_, _, elements) => format_map(elements.iter(), "HashMap { ", " }"),
        // custom types
        Value::Custom(ty, data) => format_custom(*ty, data),
    }
}

pub fn format_fields(fields: &Fields) -> Result<String, DecodeError> {
    match fields {
        Fields::Named(named) => format_vec(named.iter(), "{ ", " }"),
        Fields::Unnamed(unnamed) => format_vec(unnamed.iter(), "( ", " )"),
        Fields::Unit => Ok(String::from("()")),
    }
}

pub fn format_vec<'a, I: Iterator<Item = &'a Value>>(
    itr: I,
    begin: &str,
    end: &str,
) -> Result<String, DecodeError> {
    let mut buf = String::from(begin);
    for (i, x) in itr.enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(format_value(x)?.as_str());
    }
    buf.push_str(end);
    Ok(buf)
}

pub fn format_map<'a, I: Iterator<Item = &'a (Value, Value)>>(
    itr: I,
    begin: &str,
    end: &str,
) -> Result<String, DecodeError> {
    let mut buf = String::from(begin);
    for (i, x) in itr.enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(format!("{} => {}", format_value(&x.0)?, format_value(&x.1)?).as_str());
    }
    buf.push_str(end);
    Ok(buf)
}

pub fn format_custom(ty: u8, data: &[u8]) -> Result<String, DecodeError> {
    match ty {
        SCRYPTO_TYPE_AMOUNT => {
            let amount = Amount::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            Ok(format!("Amount({})", amount))
        }
        SCRYPTO_TYPE_ADDRESS => {
            let address =
                Address::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            Ok(format!("Address({})", address))
        }
        SCRYPTO_TYPE_H256 => {
            let h256 = H256::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            Ok(format!("H256({})", h256))
        }
        SCRYPTO_TYPE_SID => {
            let sid = SID::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            Ok(format!("SID({})", sid))
        }
        SCRYPTO_TYPE_BID => {
            let bid = BID::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            Ok(format!("BID({})", bid))
        }
        SCRYPTO_TYPE_RID => {
            let rid = RID::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            Ok(format!("RID({})", rid))
        }
        SCRYPTO_TYPE_VID => {
            let vid = VID::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            Ok(format!("VID({})", vid))
        }
        _ => Err(DecodeError::InvalidType {
            expected: None,
            actual: ty,
        }),
    }
}
