use sbor::any::*;
use sbor::*;
use scrypto::constants::*;
use scrypto::rust::borrow::Borrow;
use scrypto::rust::convert::TryFrom;
use scrypto::rust::format;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::ledger::*;

/// Format SBOR data.
pub fn format_sbor(data: &[u8]) -> Result<String, DecodeError> {
    let ledger = InMemoryLedger::new();
    let mut vaults = vec![];
    format_sbor_with_ledger(data, &ledger, &mut vaults)
}

/// Format SBOR data, using ledger state.
pub fn format_sbor_with_ledger<L: Ledger>(
    data: &[u8],
    ledger: &L,
    vaults: &mut Vec<VID>,
) -> Result<String, DecodeError> {
    let value = decode_any(data)?;
    format_value(&value, ledger, vaults)
}

pub fn format_value<L: Ledger>(
    value: &Value,
    ledger: &L,
    vaults: &mut Vec<VID>,
) -> Result<String, DecodeError> {
    match value {
        // primitive types
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
        // struct & enum
        Value::Struct(fields) => Ok(format!("Struct {}", format_fields(fields, ledger, vaults)?)),
        Value::Enum(index, fields) => Ok(format!(
            "Enum::{} {}",
            index,
            format_fields(fields, ledger, vaults)?
        )),
        // rust types
        Value::Option(v) => match v.borrow() {
            Some(x) => Ok(format!("Some({})", format_value(x, ledger, vaults)?)),
            None => Ok(String::from("None")),
        },
        Value::Box(v) => Ok(format!(
            "Box({})",
            format_value(v.borrow(), ledger, vaults)?
        )),
        Value::Array(_, elements) => format_vec(elements.iter(), "[", "]", ledger, vaults),
        Value::Tuple(elements) => format_vec(elements.iter(), "(", ")", ledger, vaults),
        Value::Result(v) => match v.borrow() {
            Ok(x) => Ok(format!("Ok({})", format_value(x, ledger, vaults)?)),
            Err(x) => Ok(format!("Err({})", format_value(x, ledger, vaults)?)),
        },
        // collections
        Value::Vec(_, elements) => format_vec(elements.iter(), "Vec { ", " }", ledger, vaults),
        Value::TreeSet(_, elements) => {
            format_vec(elements.iter(), "TreeSet { ", " }", ledger, vaults)
        }
        Value::HashSet(_, elements) => {
            format_vec(elements.iter(), "HashSet { ", " }", ledger, vaults)
        }
        Value::TreeMap(_, _, elements) => {
            format_map(elements.iter(), "TreeMap { ", " }", ledger, vaults)
        }
        Value::HashMap(_, _, elements) => {
            format_map(elements.iter(), "HashMap { ", " }", ledger, vaults)
        }
        // custom types
        Value::Custom(ty, data) => format_custom(*ty, data, ledger, vaults),
    }
}

pub fn format_fields<L: Ledger>(
    fields: &Fields,
    ledger: &L,
    vaults: &mut Vec<VID>,
) -> Result<String, DecodeError> {
    match fields {
        Fields::Named(named) => format_vec(named.iter(), "{ ", " }", ledger, vaults),
        Fields::Unnamed(unnamed) => format_vec(unnamed.iter(), "( ", " )", ledger, vaults),
        Fields::Unit => Ok(String::from("()")),
    }
}

pub fn format_vec<'a, L: Ledger, I: Iterator<Item = &'a Value>>(
    itr: I,
    begin: &str,
    end: &str,
    ledger: &L,
    vaults: &mut Vec<VID>,
) -> Result<String, DecodeError> {
    let mut buf = String::from(begin);
    for (i, x) in itr.enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(format_value(x, ledger, vaults)?.as_str());
    }
    buf.push_str(end);
    Ok(buf)
}

pub fn format_map<'a, L: Ledger, I: Iterator<Item = &'a (Value, Value)>>(
    itr: I,
    begin: &str,
    end: &str,
    ledger: &L,
    vaults: &mut Vec<VID>,
) -> Result<String, DecodeError> {
    let mut buf = String::from(begin);
    for (i, x) in itr.enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(
            format!(
                "{} => {}",
                format_value(&x.0, ledger, vaults)?,
                format_value(&x.1, ledger, vaults)?
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
    ledger: &L,
    vaults: &mut Vec<VID>,
) -> Result<String, DecodeError> {
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
        SCRYPTO_TYPE_MID => {
            let mid = MID::try_from(data).map_err(|_| DecodeError::InvalidCustomData(ty))?;

            let mut buf = String::new();
            if let Some(lazy_map) = ledger.get_lazy_map(mid) {
                for (i, (k, v)) in lazy_map.map().iter().enumerate() {
                    if i != 0 {
                        buf.push_str(", ");
                    }
                    buf.push_str(format_sbor_with_ledger(k, ledger, vaults)?.as_str());
                    buf.push_str(" => ");
                    buf.push_str(format_sbor_with_ledger(v, ledger, vaults)?.as_str());
                }
            };

            Ok(format!("MID({}) {{ {} }}", mid, buf))
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
            vaults.push(vid);
            Ok(format!("VID({})", vid))
        }
        _ => Err(DecodeError::InvalidType {
            expected: None,
            actual: ty,
        }),
    }
}
