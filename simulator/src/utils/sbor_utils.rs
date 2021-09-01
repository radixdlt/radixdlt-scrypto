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
    fmt_value(&value, ledger, res)
}

fn fmt_value<L: Ledger>(
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
        Value::Option(v) => match v {
            Some(x) => Ok(format!("Some({})", fmt_value(x.borrow(), le, res)?)),
            None => Ok(String::from("None")),
        },
        Value::Box(v) => Ok(format!("Box({})", fmt_value(v.borrow(), le, res)?)),
        Value::Array(elements) => fmt_vec(elements.iter(), "[", "]", le, res),
        Value::Tuple(elements) => fmt_vec(elements.iter(), "(", ")", le, res),
        Value::Struct(fields) => Ok(format!("Struct {}", fmt_fields(fields, le, res)?)),
        Value::Enum(index, fields) => {
            Ok(format!("Enum::{} {}", index, fmt_fields(fields, le, res)?))
        }
        // collections
        Value::Vec(elements) => fmt_vec(elements.iter(), "Vec { ", " }", le, res),
        Value::TreeSet(elements) => fmt_vec(elements.iter(), "TreeSet { ", " }", le, res),
        Value::HashSet(elements) => fmt_vec(elements.iter(), "HashSet { ", " }", le, res),
        Value::TreeMap(elements) => fmt_map(elements.iter(), "TreeMap { ", " }", le, res),
        Value::HashMap(elements) => fmt_map(elements.iter(), "HashMap { ", " }", le, res),
        Value::Custom(ty, data) => fmt_custom(*ty, data, le, res),
    }
}

fn fmt_fields<L: Ledger>(
    fields: &Fields,
    le: &L,
    res: &mut Vec<Bucket>,
) -> Result<String, DecodeError> {
    match fields {
        Fields::Named(named) => fmt_vec(named.iter(), "{ ", " }", le, res),
        Fields::Unnamed(unnamed) => fmt_vec(unnamed.iter(), "( ", " )", le, res),
        Fields::Unit => Ok(String::from("()")),
    }
}

fn fmt_vec<'a, I: Iterator<Item = &'a Value>, L: Ledger>(
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
        buf.push_str(fmt_value(x, le, res)?.as_str());
    }
    buf.push_str(end);
    Ok(buf)
}

fn fmt_map<'a, I: Iterator<Item = &'a (Value, Value)>, L: Ledger>(
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
                fmt_value(&x.0, le, res)?,
                fmt_value(&x.1, le, res)?
            )
            .as_str(),
        );
    }
    buf.push_str(end);
    Ok(buf)
}

fn fmt_custom<L: Ledger>(
    ty: u8,
    data: &[u8],
    le: &L,
    res: &mut Vec<Bucket>,
) -> Result<String, DecodeError> {
    match ty {
        SCRYPTO_TYPE_U256 => Ok(<U256>::from_little_endian(data).to_string()),
        SCRYPTO_TYPE_ADDRESS => Ok(format!("Address ({})", decode_custom::<Address>(ty, data)?)),
        SCRYPTO_TYPE_H256 => Ok(format!("H256 ({})", decode_custom::<Address>(ty, data)?)),
        SCRYPTO_TYPE_MID => {
            let mid = decode_custom::<MID>(ty, data)?;
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
        SCRYPTO_TYPE_RID => Ok(format!("RID ({})", decode_custom::<Address>(ty, data)?)),
        SCRYPTO_TYPE_BID => {
            let bid = decode_custom::<BID>(ty, data)?;
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
        _ => Err(DecodeError::InvalidType {
            expected: 0xff,
            actual: ty,
        }),
    }
}

fn decode_custom<'a, T: TryFrom<&'a [u8]> + ToString>(
    ty: u8,
    slice: &'a [u8],
) -> Result<T, DecodeError> {
    <T>::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(ty))
}
