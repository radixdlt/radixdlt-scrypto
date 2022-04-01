use sbor::type_id::*;
use sbor::{any::*, *};

use crate::buffer::*;
use crate::component::*;
use crate::crypto::*;
use crate::engine::types::*;
use crate::math::*;
use crate::resource::*;
use crate::rust::borrow::Borrow;
use crate::rust::collections::HashMap;
use crate::rust::fmt;
use crate::rust::format;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents an error when parsing a Scrypto value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseScryptoValueError {
    DecodeError(DecodeError),
    CustomValueCheckError(ScryptoCustomValueCheckError),
}

/// A Scrypto value is a SBOR value of which the custom types are the ones defined by `ScryptoType`.
#[derive(Clone, PartialEq, Eq)]
pub struct ScryptoValue {
    pub raw: Vec<u8>,
    pub dom: Value,
    pub bucket_ids: Vec<BucketId>,
    pub proof_ids: Vec<ProofId>,
    pub vault_ids: Vec<VaultId>,
    pub lazy_map_ids: Vec<LazyMapId>,
}

impl ScryptoValue {
    pub fn from_slice(slice: &[u8]) -> Result<Self, ParseScryptoValueError> {
        // Decode with SBOR
        let value = decode_any(slice).map_err(ParseScryptoValueError::DecodeError)?;

        // Scrypto specific types checking
        let mut checker = ScryptoCustomValueChecker::new();
        traverse_any(&value, &mut checker)
            .map_err(ParseScryptoValueError::CustomValueCheckError)?;

        Ok(ScryptoValue {
            raw: slice.to_vec(),
            dom: value,
            bucket_ids: checker.buckets.iter().map(|e| e.0).collect(),
            proof_ids: checker.proofs.iter().map(|e| e.0).collect(),
            vault_ids: checker.vaults.iter().map(|e| e.0).collect(),
            lazy_map_ids: checker.lazy_maps.iter().map(|e| e.id).collect(),
        })
    }

    pub fn from_value<T: Encode>(value: &T) -> Self {
        ScryptoValue::from_slice(&scrypto_encode(value)).unwrap()
    }

    pub fn to_string(&self) -> String {
        if self.raw.len() <= 1024 {
            ScryptoValueFormatter::format_value(&self.dom, &HashMap::new(), &HashMap::new())
        } else {
            format!("LargeValue(length: {})", self.raw.len())
        }
    }

    pub fn to_string_with_context(
        &self,
        bucket_ids: &HashMap<BucketId, String>,
        proof_ids: &HashMap<ProofId, String>,
    ) -> String {
        if self.raw.len() <= 1024 {
            ScryptoValueFormatter::format_value(&self.dom, bucket_ids, proof_ids)
        } else {
            format!("LargeValue(length: {})", self.raw.len())
        }
    }
}

impl fmt::Debug for ScryptoValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for ScryptoValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// A checker the check a Scrypto-specific value.
pub struct ScryptoCustomValueChecker {
    pub buckets: Vec<Bucket>,
    pub proofs: Vec<Proof>,
    pub vaults: Vec<Vault>,
    pub lazy_maps: Vec<LazyMap<(), ()>>,
}

/// Represents an error when validating a Scrypto-specific value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValueCheckError {
    DecodeError(DecodeError),
    InvalidTypeId(u8),
    InvalidDecimal(ParseDecimalError),
    InvalidPackageAddress(ParsePackageAddressError),
    InvalidComponentAddress(ParseComponentAddressError),
    InvalidResourceAddress(ParseResourceAddressError),
    InvalidHash(ParseHashError),
    InvalidEcdsaPublicKey(ParseEcdsaPublicKeyError),
    InvalidEcdsaSignature(ParseEcdsaSignatureError),
    InvalidBucket(ParseBucketError),
    InvalidProof(ParseProofError),
    InvalidLazyMap(ParseLazyMapError),
    InvalidVault(ParseVaultError),
    InvalidNonFungibleId(ParseNonFungibleIdError),
    InvalidNonFungibleAddress(ParseNonFungibleAddressError),
}

impl ScryptoCustomValueChecker {
    pub fn new() -> Self {
        Self {
            buckets: Vec::new(),
            proofs: Vec::new(),
            vaults: Vec::new(),
            lazy_maps: Vec::new(),
        }
    }
}

impl CustomValueVisitor for ScryptoCustomValueChecker {
    type Err = ScryptoCustomValueCheckError;

    fn visit(&mut self, type_id: u8, data: &[u8]) -> Result<(), Self::Err> {
        match ScryptoType::from_id(type_id).ok_or(Self::Err::InvalidTypeId(type_id))? {
            ScryptoType::PackageAddress => {
                PackageAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidPackageAddress)?;
            }
            ScryptoType::ComponentAddress => {
                ComponentAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidComponentAddress)?;
            }
            ScryptoType::LazyMap => {
                self.lazy_maps.push(
                    LazyMap::try_from(data)
                        .map_err(ScryptoCustomValueCheckError::InvalidLazyMap)?,
                );
            }
            ScryptoType::Hash => {
                Hash::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidHash)?;
            }
            ScryptoType::EcdsaPublicKey => {
                EcdsaPublicKey::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidEcdsaPublicKey)?;
            }
            ScryptoType::EcdsaSignature => {
                EcdsaSignature::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidEcdsaSignature)?;
            }
            ScryptoType::Decimal => {
                Decimal::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidDecimal)?;
            }
            ScryptoType::Bucket => {
                self.buckets.push(
                    Bucket::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidBucket)?,
                );
            }
            ScryptoType::Proof => {
                self.proofs.push(
                    Proof::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidProof)?,
                );
            }
            ScryptoType::Vault => {
                self.vaults.push(
                    Vault::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidVault)?,
                );
            }
            ScryptoType::NonFungibleId => {
                NonFungibleId::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidNonFungibleId)?;
            }
            ScryptoType::NonFungibleAddress => {
                NonFungibleAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidNonFungibleAddress)?;
            }
            ScryptoType::ResourceAddress => {
                ResourceAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidResourceAddress)?;
            }
        }
        Ok(())
    }
}

pub struct ScryptoValueFormatter {}

impl ScryptoValueFormatter {
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
                format!(
                    "Struct({})",
                    Self::format_elements(fields, bucket_ids, proof_ids)
                )
            }
            Value::Enum(index, fields) => {
                format!(
                    "Enum({}u8{}{})",
                    index,
                    if fields.is_empty() { "" } else { ", " },
                    Self::format_elements(fields, bucket_ids, proof_ids)
                )
            }
            // rust types
            Value::Option(v) => match v.borrow() {
                Some(x) => format!("Some({})", Self::format_value(x, bucket_ids, proof_ids)),
                None => "None".to_string(),
            },
            Value::Array(kind, elements) => format!(
                "Array<{}>({})",
                Self::format_kind(*kind),
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            Value::Tuple(elements) => format!(
                "Tuple({})",
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            Value::Result(v) => match v.borrow() {
                Ok(x) => format!("Ok({})", Self::format_value(x, bucket_ids, proof_ids)),
                Err(x) => format!("Err({})", Self::format_value(x, bucket_ids, proof_ids)),
            },
            // collections
            Value::Vec(kind, elements) => {
                format!(
                    "Vec<{}>({})",
                    Self::format_kind(*kind),
                    Self::format_elements(elements, bucket_ids, proof_ids)
                )
            }
            Value::TreeSet(kind, elements) => format!(
                "TreeSet<{}>({})",
                Self::format_kind(*kind),
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            Value::HashSet(kind, elements) => format!(
                "HashSet<{}>({})",
                Self::format_kind(*kind),
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            Value::TreeMap(key, value, elements) => format!(
                "TreeMap<{}, {}>({})",
                Self::format_kind(*key),
                Self::format_kind(*value),
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            Value::HashMap(key, value, elements) => format!(
                "HashMap<{}, {}>({})",
                Self::format_kind(*key),
                Self::format_kind(*value),
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            // custom types
            Value::Custom(kind, data) => {
                Self::from_custom_value(*kind, data, bucket_ids, proof_ids)
            }
        }
    }

    pub fn format_kind(kind: u8) -> String {
        if let Some(ty) = ScryptoType::from_id(kind) {
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
            buf.push_str(Self::format_value(x, bucket_ids, proof_ids).as_str());
        }
        buf
    }
    pub fn from_custom_value(
        type_id: u8,
        data: &[u8],
        bucket_ids: &HashMap<BucketId, String>,
        proof_ids: &HashMap<ProofId, String>,
    ) -> String {
        match ScryptoType::from_id(type_id).unwrap() {
            ScryptoType::Decimal => format!("Decimal(\"{}\")", Decimal::try_from(data).unwrap()),
            ScryptoType::PackageAddress => {
                format!(
                    "PackageAddress(\"{}\")",
                    PackageAddress::try_from(data).unwrap()
                )
            }
            ScryptoType::ComponentAddress => {
                format!(
                    "ComponentAddress(\"{}\")",
                    ComponentAddress::try_from(data).unwrap()
                )
            }
            ScryptoType::LazyMap => format!(
                "LazyMap(\"{}\")",
                LazyMap::<(), ()>::try_from(data).unwrap()
            ),
            ScryptoType::Hash => format!("Hash(\"{}\")", Hash::try_from(data).unwrap()),
            ScryptoType::EcdsaPublicKey => {
                format!(
                    "EcdsaPublicKey(\"{}\")",
                    EcdsaPublicKey::try_from(data).unwrap()
                )
            }
            ScryptoType::EcdsaSignature => {
                format!(
                    "EcdsaSignature(\"{}\")",
                    EcdsaSignature::try_from(data).unwrap()
                )
            }
            ScryptoType::Bucket => {
                let bucket = Bucket::try_from(data).unwrap();
                if let Some(name) = bucket_ids.get(&bucket.0) {
                    format!("Bucket(\"{}\")", name)
                } else {
                    format!("Bucket({}u32)", bucket.0)
                }
            }
            ScryptoType::Proof => {
                let proof = Proof::try_from(data).unwrap();
                if let Some(name) = proof_ids.get(&proof.0) {
                    format!("Proof(\"{}\")", name)
                } else {
                    format!("Proof({}u32)", proof.0)
                }
            }
            ScryptoType::Vault => format!("Vault(\"{}\")", Vault::try_from(data).unwrap()),
            ScryptoType::NonFungibleId => format!(
                "NonFungibleId(\"{}\")",
                NonFungibleId::try_from(data).unwrap()
            ),
            ScryptoType::NonFungibleAddress => format!(
                "NonFungibleAddress(\"{}\")",
                NonFungibleAddress::try_from(data).unwrap()
            ),
            ScryptoType::ResourceAddress => format!(
                "ResourceAddress(\"{}\")",
                ResourceAddress::try_from(data).unwrap()
            ),
        }
    }
}
