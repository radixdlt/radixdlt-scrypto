use sbor::any::*;
use sbor::type_id::*;
use sbor::Encode;

use scrypto::buffer::scrypto_encode;
use scrypto::engine::types::*;
use scrypto::rust::borrow::Borrow;
use scrypto::rust::collections::HashMap;
use scrypto::rust::fmt;
use scrypto::rust::format;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::errors::DataValidationError;

#[derive(Clone, PartialEq, Eq)]
pub struct ValidatedData {
    pub raw: Vec<u8>,
    pub dom: Value,
    pub bucket_ids: Vec<BucketId>,
    pub proof_ids: Vec<ProofId>,
    pub vault_ids: Vec<VaultId>,
    pub lazy_map_ids: Vec<LazyMapId>,
}

impl ValidatedData {
    pub fn from_slice(slice: &[u8]) -> Result<Self, DataValidationError> {
        // SBOR basic validation
        let value = decode_any(slice).map_err(DataValidationError::DecodeError)?;

        // Additional custom value validation
        let mut validator = CustomValueValidator::new();
        traverse_any(&value, &mut validator)
            .map_err(DataValidationError::CustomValueValidatorError)?;

        Ok(ValidatedData {
            raw: slice.to_vec(),
            dom: value,
            bucket_ids: validator.buckets.iter().map(|e| e.0).collect(),
            proof_ids: validator.proofs.iter().map(|e| e.0).collect(),
            vault_ids: validator.vaults.iter().map(|e| e.0).collect(),
            lazy_map_ids: validator.lazy_maps.iter().map(|e| e.id).collect(),
        })
    }

    pub fn from_value<T: Encode>(value: &T) -> Self {
        ValidatedData::from_slice(&scrypto_encode(value)).unwrap()
    }
}

impl fmt::Debug for ValidatedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.raw.len() <= 1024 {
            write!(
                f,
                "{}",
                format_value(&self.dom, &HashMap::new(), &HashMap::new())
            )
        } else {
            write!(f, "LargeValue(len: {})", self.raw.len())
        }
    }
}

impl fmt::Display for ValidatedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn format_value(
    value: &Value,
    bucket_ids: &HashMap<BucketId, String>,
    proof_ids: &HashMap<ProofId, String>,
) -> String {
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
        Value::Struct(fields) => {
            format!("Struct({})", format_elements(fields, bucket_ids, proof_ids))
        }
        Value::Enum(index, fields) => {
            format!(
                "Enum({}u8{}{})",
                index,
                if fields.is_empty() { "" } else { ", " },
                format_elements(fields, bucket_ids, proof_ids)
            )
        }
        // rust types
        Value::Option(v) => match v.borrow() {
            Some(x) => format!("Some({})", format_value(x, bucket_ids, proof_ids)),
            None => "None".to_string(),
        },
        Value::Array(kind, elements) => format!(
            "Array<{}>({})",
            format_kind(*kind),
            format_elements(elements, bucket_ids, proof_ids)
        ),
        Value::Tuple(elements) => format!(
            "Tuple({})",
            format_elements(elements, bucket_ids, proof_ids)
        ),
        Value::Result(v) => match v.borrow() {
            Ok(x) => format!("Ok({})", format_value(x, bucket_ids, proof_ids)),
            Err(x) => format!("Err({})", format_value(x, bucket_ids, proof_ids)),
        },
        // collections
        Value::Vec(kind, elements) => {
            format!(
                "Vec<{}>({})",
                format_kind(*kind),
                format_elements(elements, bucket_ids, proof_ids)
            )
        }
        Value::TreeSet(kind, elements) => format!(
            "TreeSet<{}>({})",
            format_kind(*kind),
            format_elements(elements, bucket_ids, proof_ids)
        ),
        Value::HashSet(kind, elements) => format!(
            "HashSet<{}>({})",
            format_kind(*kind),
            format_elements(elements, bucket_ids, proof_ids)
        ),
        Value::TreeMap(key, value, elements) => format!(
            "TreeMap<{}, {}>({})",
            format_kind(*key),
            format_kind(*value),
            format_elements(elements, bucket_ids, proof_ids)
        ),
        Value::HashMap(key, value, elements) => format!(
            "HashMap<{}, {}>({})",
            format_kind(*key),
            format_kind(*value),
            format_elements(elements, bucket_ids, proof_ids)
        ),
        // custom types
        Value::Custom(kind, data) => {
            CustomValueFormatter::format(*kind, data, bucket_ids, proof_ids)
        }
    }
}

pub fn format_kind(kind: u8) -> String {
    if let Some(ty) = CustomType::from_id(kind) {
        return ty.name();
    }

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
        TYPE_ARRAY => "Array",
        TYPE_TUPLE => "Tuple",
        TYPE_RESULT => "Result",
        // collections
        TYPE_VEC => "Vec",
        TYPE_TREE_SET => "TreeSet",
        TYPE_TREE_MAP => "TreeMap",
        TYPE_HASH_SET => "HashSet",
        TYPE_HASH_MAP => "HashMap",
        //
        _ => panic!("Illegal state"),
    }
    .to_string()
}

pub fn format_elements(
    values: &[Value],
    bucket_ids: &HashMap<BucketId, String>,
    proof_ids: &HashMap<ProofId, String>,
) -> String {
    let mut buf = String::new();
    for (i, x) in values.iter().enumerate() {
        if i != 0 {
            buf.push_str(", ");
        }
        buf.push_str(format_value(x, bucket_ids, proof_ids).as_str());
    }
    buf
}
