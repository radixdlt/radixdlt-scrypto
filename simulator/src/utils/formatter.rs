use radix_engine::ledger::*;
use radix_engine::model::*;
use sbor::parse::*;
use sbor::*;
use scrypto::constants::*;
use scrypto::rust::borrow::Borrow;
use scrypto::rust::convert::TryFrom;
use scrypto::types::*;

pub fn format_sbor<L: Ledger>(
    data: &[u8],
    ledger: &L,
    res: &mut Vec<Bucket>,
) -> Result<String, DecodeError> {
    let value = parse_any(data)?;
    format_value(&value, ledger, res)
}

pub fn format_value<L: Ledger>(
    value: &Value,
    le: &L,
    res: &mut Vec<Bucket>,
) -> Result<String, DecodeError> {
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
            Some(x) => Ok(format!("Some({})", format_value(x, le, res)?)),
            None => Ok(String::from("None")),
        },
        Value::Box(v) => Ok(format!("Box({})", format_value(v.borrow(), le, res)?)),
        Value::Array(_, elements) => format_vec(elements.iter(), "[", "]", le, res),
        Value::Tuple(elements) => format_vec(elements.iter(), "(", ")", le, res),
        Value::Struct(fields) => Ok(format!("Struct {}", format_fields(fields, le, res)?)),
        Value::Enum(index, fields) => Ok(format!(
            "Enum::{} {}",
            index,
            format_fields(fields, le, res)?
        )),
        // collections
        Value::Vec(_, elements) => format_vec(elements.iter(), "Vec { ", " }", le, res),
        Value::TreeSet(_, elements) => format_vec(elements.iter(), "TreeSet { ", " }", le, res),
        Value::HashSet(_, elements) => format_vec(elements.iter(), "HashSet { ", " }", le, res),
        Value::TreeMap(_, _, elements) => format_map(elements.iter(), "TreeMap { ", " }", le, res),
        Value::HashMap(_, _, elements) => format_map(elements.iter(), "HashMap { ", " }", le, res),
        // custom types
        Value::Custom(ty, data) => format_custom(*ty, data, le, res),
    }
}

pub fn format_fields<L: Ledger>(
    fields: &Fields,
    le: &L,
    res: &mut Vec<Bucket>,
) -> Result<String, DecodeError> {
    match fields {
        Fields::Named(named) => format_vec(named.iter(), "{ ", " }", le, res),
        Fields::Unnamed(unnamed) => format_vec(unnamed.iter(), "( ", " )", le, res),
        Fields::Unit => Ok(String::from("()")),
    }
}

pub fn format_vec<'a, I: Iterator<Item = &'a Value>, L: Ledger>(
    itr: I,
    begin: &str,
    end: &str,
    le: &L,
    res: &mut Vec<Bucket>,
) -> Result<String, DecodeError> {
    let mut buf = String::from(begin);
    for (i, x) in itr.enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(format_value(x, le, res)?.as_str());
    }
    buf.push_str(end);
    Ok(buf)
}

pub fn format_map<'a, I: Iterator<Item = &'a (Value, Value)>, L: Ledger>(
    itr: I,
    begin: &str,
    end: &str,
    le: &L,
    res: &mut Vec<Bucket>,
) -> Result<String, DecodeError> {
    let mut buf = String::from(begin);
    for (i, x) in itr.enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(
            format!(
                "{} => {}",
                format_value(&x.0, le, res)?,
                format_value(&x.1, le, res)?
            )
            .as_str(),
        );
    }
    buf.push_str(end);
    Ok(buf)
}

pub fn format_custom<L: Ledger>(
    ty: u8,
    data: &[u8],
    le: &L,
    res: &mut Vec<Bucket>,
) -> Result<String, DecodeError> {
    match ty {
        SCRYPTO_TYPE_U256 => Ok(<U256>::from_little_endian(data).to_string()),
        SCRYPTO_TYPE_ADDRESS => {
            let address =
                Address::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            Ok(format!("Address ({})", address))
        }
        SCRYPTO_TYPE_H256 => {
            let h256 = H256::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            Ok(format!("H256 ({})", h256))
        }
        SCRYPTO_TYPE_MID => {
            let mid = MID::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            let map = le.get_map(mid).ok_or(DecodeError::InvalidCustomData(ty))?;
            let mut buf = String::from("s");
            for (i, (k, v)) in map.map.iter().enumerate() {
                if i != 0 {
                    buf.push_str(", ");
                }
                buf.push_str(format_sbor(&k, le, res)?.as_str());
                buf.push_str(" => ");
                buf.push_str(format_sbor(&v, le, res)?.as_str());
            }
            Ok(format!("Map {{ mid: {}, entries: [{}] }}", mid, buf))
        }
        SCRYPTO_TYPE_BID => {
            let bid = BID::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            let bucket = le
                .get_bucket(bid)
                .ok_or(DecodeError::InvalidCustomData(ty))?;
            res.push(Bucket::new(bucket.amount(), bucket.resource()));
            Ok(format!(
                "Bucket {{ bid: {}, amount: {}, resource: {} }}",
                bid,
                bucket.amount(),
                bucket.resource()
            ))
        }
        SCRYPTO_TYPE_RID => {
            let rid = RID::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;
            Ok(format!("RID ({})", rid))
        }
        _ => Err(DecodeError::InvalidType {
            expected: 0xff,
            actual: ty,
        }),
    }
}
