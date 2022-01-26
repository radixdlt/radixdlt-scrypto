use sbor::any::*;
use sbor::type_id::*;
use sbor::*;
use scrypto::buffer::*;
use scrypto::rust::borrow::Borrow;
use scrypto::rust::convert::TryFrom;
use scrypto::rust::format;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::ledger::*;

/// Formats any data.
pub fn format_data(data: &[u8]) -> Result<String, DecodeError> {
    let ledger = InMemoryLedger::new();
    let mut vaults = vec![];
    format_data_with_ledger(data, &ledger, &mut vaults)
}

/// Formats any data, using ledger state.
pub fn format_data_with_ledger<L: Ledger>(
    data: &[u8],
    ledger: &L,
    vaults: &mut Vec<Vid>,
) -> Result<String, DecodeError> {
    let value = decode_any(data)?;
    Ok(format_value(&value))
}

pub fn format_value(value: &Value) -> String {
    match value {
        // primitive types
        Value::Unit => "()".to_string(),
        Value::Bool(v) => v.to_string(),
        Value::I8(v) => format!("{}i8", v),
        Value::I16(v) => format!("{}i16", v),
        Value::I32(v) => format!("{}i32", v),
        Value::I64(v) => format!("{}i64", v),
        Value::I128(v) => format!("{}i128", v),
        Value::U8(v) => format!("{}u8", v),
        Value::U16(v) => format!("{}u16", v),
        Value::U32(v) => format!("{}u32", v),
        Value::U64(v) => format!("{}u64", v),
        Value::U128(v) => format!("{}u128", v),
        Value::String(v) => format!("\"{}\"", v),
        // struct & enum
        Value::Struct(fields) => format!("Struct({})", format_fields(fields)),
        Value::Enum(index, fields) => {
            let fields = format_fields(fields);
            format!(
                "Enum({}u8{}{})",
                index,
                if fields.is_empty() { "" } else { ", " },
                fields
            )
        }
        // rust types
        Value::Option(v) => match v.borrow() {
            Some(x) => format!("Some({})", format_value(x)),
            None => "None".to_string(),
        },
        Value::Box(v) => format!("Box({})", format_value(v.borrow())),
        Value::Array(kind, elements) => format!(
            "Array<{}>({})",
            format_kind(*kind),
            format_elements(elements)
        ),
        Value::Tuple(elements) => format!("Tuple({})", format_elements(elements)),
        Value::Result(v) => match v.borrow() {
            Ok(x) => format!("Ok({})", format_value(x)),
            Err(x) => format!("Err({})", format_value(x)),
        },
        // collections
        Value::Vec(kind, elements) => {
            format!("Vec<{}>({})", format_kind(*kind), format_elements(elements))
        }
        Value::TreeSet(kind, elements) => format!(
            "TreeSet<{}>({})",
            format_kind(*kind),
            format_elements(elements)
        ),
        Value::HashSet(kind, elements) => format!(
            "HashSet<{}>({})",
            format_kind(*kind),
            format_elements(elements)
        ),
        Value::TreeMap(key, value, elements) => format!(
            "TreeMap<{}, {}>({})",
            format_kind(*key),
            format_kind(*value),
            format_elements(elements)
        ),
        Value::HashMap(key, value, elements) => format!(
            "HashMap<{}, {}>({})",
            format_kind(*key),
            format_kind(*value),
            format_elements(elements)
        ),
        // custom types
        Value::Custom(kind, data) => format_custom(*kind, data),
    }
}

pub fn format_kind(kind: u8) -> String {
    match kind {
        // primitive types
        TYPE_UNIT => "Unit",
        TYPE_BOOL => "Bool",
        TYPE_I8 => "I8",
        TYPE_I16 => "I16",
        TYPE_I32 => "I32",
        TYPE_I64 => "I64",
        TYPE_I128 => "I128",
        TYPE_U8 => "U8",
        TYPE_U16 => "U16",
        TYPE_U32 => "U32",
        TYPE_U64 => "U64",
        TYPE_U128 => "U128",
        TYPE_STRING => "String",
        // struct & enum
        TYPE_STRUCT => "Struct",
        TYPE_ENUM => "Enum",
        TYPE_OPTION => "Option",
        TYPE_BOX => "Box",
        TYPE_ARRAY => "Array",
        TYPE_TUPLE => "Tuple",
        TYPE_RESULT => "Result",
        // collections
        TYPE_VEC => "Vec",
        TYPE_TREE_SET => "TreeSet",
        TYPE_TREE_MAP => "TreeMap",
        TYPE_HASH_SET => "HashSet",
        TYPE_HASH_MAP => "HashMap",
        // scrypto
        SCRYPTO_TYPE_DECIMAL => "Decimal",
        SCRYPTO_TYPE_BIG_DECIMAL => "BigDecimal",
        SCRYPTO_TYPE_ADDRESS => "Address",
        SCRYPTO_TYPE_H256 => "Hash",
        SCRYPTO_TYPE_BID => "Bucket",
        SCRYPTO_TYPE_RID => "BucketRef",
        SCRYPTO_TYPE_MID => "LazyMap",
        SCRYPTO_TYPE_VID => "Vault",
        _ => panic!("Illegal state"),
    }
    .to_string()
}

pub fn format_fields(fields: &Fields) -> String {
    match fields {
        Fields::Named(named) => format!("{{{}}}", format_elements(named)),
        Fields::Unnamed(unnamed) => {
            format!("({})", format_elements(unnamed))
        }
        Fields::Unit => "".into(),
    }
}

pub fn format_elements(values: &[Value]) -> String {
    let mut buf = String::new();
    for (i, x) in values.iter().enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(format_value(x).as_str());
    }
    buf
}

pub fn format_custom(ty: u8, data: &[u8]) -> String {
    match ty {
        SCRYPTO_TYPE_DECIMAL => format!("Decimal({})", Decimal::try_from(data).unwrap()),
        SCRYPTO_TYPE_BIG_DECIMAL => format!("BigDecimal({})", BigDecimal::try_from(data).unwrap()),
        SCRYPTO_TYPE_ADDRESS => format!("Address({})", Address::try_from(data).unwrap()),
        SCRYPTO_TYPE_H256 => format!("Hash({})", H256::try_from(data).unwrap()),
        SCRYPTO_TYPE_MID => format!("LazyMap({})", Mid::try_from(data).unwrap()),
        SCRYPTO_TYPE_BID => format!("Bucket({}u32)", Bid::try_from(data).unwrap().0),
        SCRYPTO_TYPE_RID => format!("BucketRef({}u32)", Rid::try_from(data).unwrap().0),
        SCRYPTO_TYPE_VID => format!("Vault({})", Vid::try_from(data).unwrap()),
        _ => panic!("Illegal state"),
    }
}
