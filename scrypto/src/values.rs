use sbor::path::{MutableSborPath, SborPath};
use sbor::rust::borrow::Borrow;
use sbor::rust::collections::HashMap;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::type_id::*;
use sbor::{any::*, *};

use crate::abi::*;
use crate::buffer::*;
use crate::component::*;
use crate::crypto::*;
use crate::engine::types::*;
use crate::math::*;
use crate::resource::*;

/// Represents an error when parsing a Scrypto value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseScryptoValueError {
    DecodeError(DecodeError),
    CustomValueCheckError(ScryptoCustomValueCheckError),
}

pub enum ScryptoValueReplaceError {
    ProofIdNotFound(ProofId),
    BucketIdNotFound(BucketId),
}

/// A Scrypto value is a SBOR value of which the custom types are the ones defined by `ScryptoType`.
#[derive(Clone, PartialEq, Eq)]
pub struct ScryptoValue {
    pub raw: Vec<u8>,
    pub dom: Value,
    pub bucket_ids: HashMap<BucketId, SborPath>,
    pub proof_ids: HashMap<ProofId, SborPath>,
    pub vault_ids: HashSet<VaultId>,
    pub kv_store_ids: HashSet<KeyValueStoreId>,
}

// FIXME: encode as the original type, rather than Vec<u8>

impl TypeId for ScryptoValue {
    fn type_id() -> u8 {
        Vec::<u8>::type_id()
    }
}

impl Encode for ScryptoValue {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.raw.encode_value(encoder);
    }
}

impl Decode for ScryptoValue {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(Self::from_slice(&Vec::<u8>::decode_value(decoder)?)
            .expect("FIXME support untrusted ScryptoValue decoding"))
    }
}

impl ScryptoValue {
    pub fn unit() -> Self {
        Self::from_value(&())
    }

    pub fn from_value<T: Encode>(value: &T) -> Self {
        ScryptoValue::from_slice(&scrypto_encode(value)).unwrap()
    }

    pub fn from_any(value: &Value) -> Result<Self, ParseScryptoValueError> {
        ScryptoValue::from_slice(&encode_any(value))
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, ParseScryptoValueError> {
        // Decode with SBOR
        let value = decode_any(slice).map_err(ParseScryptoValueError::DecodeError)?;

        // Scrypto specific types checking
        let mut checker = ScryptoCustomValueChecker::new();
        traverse_any(&mut MutableSborPath::new(), &value, &mut checker)
            .map_err(ParseScryptoValueError::CustomValueCheckError)?;

        Ok(ScryptoValue {
            raw: slice.to_vec(),
            dom: value,
            bucket_ids: checker
                .buckets
                .drain()
                .map(|(e, path)| (e.0, path))
                .collect(),
            proof_ids: checker
                .proofs
                .drain()
                .map(|(e, path)| (e.0, path))
                .collect(),
            vault_ids: checker.vaults.iter().map(|e| e.0).collect(),
            kv_store_ids: checker.lazy_maps.iter().map(|e| e.id).collect(),
        })
    }

    pub fn replace_ids(
        &mut self,
        proof_replacements: &mut HashMap<ProofId, ProofId>,
        bucket_replacements: &mut HashMap<BucketId, BucketId>,
    ) -> Result<(), ScryptoValueReplaceError> {
        let mut new_proof_ids = HashMap::new();
        for (proof_id, path) in self.proof_ids.drain() {
            let next_id = proof_replacements
                .remove(&proof_id)
                .ok_or(ScryptoValueReplaceError::ProofIdNotFound(proof_id))?;
            let value = path.get_from_value_mut(&mut self.dom).unwrap();
            if let Value::Custom {
                type_id: _,
                ref mut bytes,
            } = value
            {
                *bytes = scrypto::resource::Proof(next_id).to_vec();
            } else {
                panic!("Proof Id should be custom type");
            }

            new_proof_ids.insert(next_id, path);
        }
        self.proof_ids = new_proof_ids;

        let mut new_bucket_ids = HashMap::new();
        for (bucket_id, path) in self.bucket_ids.drain() {
            let next_id = bucket_replacements
                .remove(&bucket_id)
                .ok_or(ScryptoValueReplaceError::BucketIdNotFound(bucket_id))?;
            let value = path.get_from_value_mut(&mut self.dom).unwrap();
            if let Value::Custom {
                type_id: _,
                ref mut bytes,
            } = value
            {
                *bytes = scrypto::resource::Bucket(next_id).to_vec();
            } else {
                panic!("Bucket should be custom type");
            }

            new_bucket_ids.insert(next_id, path);
        }
        self.bucket_ids = new_bucket_ids;

        self.raw = encode_any(&self.dom);

        Ok(())
    }

    pub fn to_string(&self) -> String {
        ScryptoValueFormatter::format_value(&self.dom, &HashMap::new(), &HashMap::new())
    }

    pub fn to_string_with_context(
        &self,
        bucket_ids: &HashMap<BucketId, String>,
        proof_ids: &HashMap<ProofId, String>,
    ) -> String {
        ScryptoValueFormatter::format_value(&self.dom, bucket_ids, proof_ids)
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
    pub buckets: HashMap<Bucket, SborPath>,
    pub proofs: HashMap<Proof, SborPath>,
    pub vaults: HashSet<Vault>,
    pub lazy_maps: HashSet<KeyValueStore<(), ()>>,
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
    InvalidLazyMap(ParseKeyValueStoreError),
    InvalidVault(ParseVaultError),
    InvalidNonFungibleId(ParseNonFungibleIdError),
    InvalidNonFungibleAddress(ParseNonFungibleAddressError),
    DuplicateIds,
}

impl ScryptoCustomValueChecker {
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
            proofs: HashMap::new(),
            vaults: HashSet::new(),
            lazy_maps: HashSet::new(),
        }
    }
}

impl CustomValueVisitor for ScryptoCustomValueChecker {
    type Err = ScryptoCustomValueCheckError;

    fn visit(
        &mut self,
        path: &mut MutableSborPath,
        type_id: u8,
        data: &[u8],
    ) -> Result<(), Self::Err> {
        match ScryptoType::from_id(type_id).ok_or(Self::Err::InvalidTypeId(type_id))? {
            ScryptoType::PackageAddress => {
                PackageAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidPackageAddress)?;
            }
            ScryptoType::ComponentAddress => {
                ComponentAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidComponentAddress)?;
            }
            ScryptoType::KeyValueStore => {
                let map = KeyValueStore::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidLazyMap)?;
                if !self.lazy_maps.insert(map) {
                    return Err(ScryptoCustomValueCheckError::DuplicateIds);
                }
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
                let bucket =
                    Bucket::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidBucket)?;
                if self.buckets.insert(bucket, path.clone().into()).is_some() {
                    return Err(ScryptoCustomValueCheckError::DuplicateIds);
                }
            }
            ScryptoType::Proof => {
                let proof =
                    Proof::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidProof)?;
                if self.proofs.insert(proof, path.clone().into()).is_some() {
                    return Err(ScryptoCustomValueCheckError::DuplicateIds);
                }
            }
            ScryptoType::Vault => {
                let vault =
                    Vault::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidVault)?;
                if !self.vaults.insert(vault) {
                    return Err(ScryptoCustomValueCheckError::DuplicateIds);
                }
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

/// Utility that formats any Scrypto value.
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
            Value::Bool { value } => value.to_string(),
            Value::I8 { value } => format!("{}i8", value),
            Value::I16 { value } => format!("{}i16", value),
            Value::I32 { value } => format!("{}i32", value),
            Value::I64 { value } => format!("{}i64", value),
            Value::I128 { value } => format!("{}i128", value),
            Value::U8 { value } => format!("{}u8", value),
            Value::U16 { value } => format!("{}u16", value),
            Value::U32 { value } => format!("{}u32", value),
            Value::U64 { value } => format!("{}u64", value),
            Value::U128 { value } => format!("{}u128", value),
            Value::String { value } => format!("\"{}\"", value),
            // struct & enum
            Value::Struct { fields } => {
                format!(
                    "Struct({})",
                    Self::format_elements(fields, bucket_ids, proof_ids)
                )
            }
            Value::Enum { name, fields } => {
                format!(
                    "Enum(\"{}\"{}{})",
                    name,
                    if fields.is_empty() { "" } else { ", " },
                    Self::format_elements(fields, bucket_ids, proof_ids)
                )
            }
            // rust types
            Value::Option { value } => match value.borrow() {
                Some(x) => format!("Some({})", Self::format_value(x, bucket_ids, proof_ids)),
                None => "None".to_string(),
            },
            Value::Array {
                element_type_id,
                elements,
            } => format!(
                "Array<{}>({})",
                Self::format_type_id(*element_type_id),
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            Value::Tuple { elements } => format!(
                "Tuple({})",
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            Value::Result { value } => match value.borrow() {
                Ok(x) => format!("Ok({})", Self::format_value(x, bucket_ids, proof_ids)),
                Err(x) => format!("Err({})", Self::format_value(x, bucket_ids, proof_ids)),
            },
            // collections
            Value::Vec {
                element_type_id,
                elements,
            } => {
                if *element_type_id == TYPE_U8 {
                    let bytes = elements
                        .iter()
                        .map(|e| match e {
                            Value::U8 { value } => *value,
                            _ => panic!("Unexpected element value"),
                        })
                        .collect::<Vec<u8>>();
                    format!("Bytes(\"{}\")", hex::encode(bytes))
                } else {
                    format!(
                        "Vec<{}>({})",
                        Self::format_type_id(*element_type_id),
                        Self::format_elements(elements, bucket_ids, proof_ids)
                    )
                }
            }
            Value::TreeSet {
                element_type_id,
                elements,
            } => format!(
                "TreeSet<{}>({})",
                Self::format_type_id(*element_type_id),
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            Value::HashSet {
                element_type_id,
                elements,
            } => format!(
                "HashSet<{}>({})",
                Self::format_type_id(*element_type_id),
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            Value::TreeMap {
                key_type_id,
                value_type_id,
                elements,
            } => format!(
                "TreeMap<{}, {}>({})",
                Self::format_type_id(*key_type_id),
                Self::format_type_id(*value_type_id),
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            Value::HashMap {
                key_type_id,
                value_type_id,
                elements,
            } => format!(
                "HashMap<{}, {}>({})",
                Self::format_type_id(*key_type_id),
                Self::format_type_id(*value_type_id),
                Self::format_elements(elements, bucket_ids, proof_ids)
            ),
            // custom types
            Value::Custom { type_id, bytes } => {
                Self::from_custom_value(*type_id, bytes, bucket_ids, proof_ids)
            }
        }
    }

    pub fn format_type_id(type_id: u8) -> String {
        if let Some(ty) = ScryptoType::from_id(type_id) {
            return ty.name();
        }

        match type_id {
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
            ScryptoType::KeyValueStore => format!(
                "LazyMap(\"{}\")",
                KeyValueStore::<(), ()>::try_from(data).unwrap()
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::rust::vec;
    use super::*;

    #[test]
    fn should_reject_duplicate_ids() {
        let buckets = scrypto_encode(&vec![
            scrypto::resource::Bucket(0),
            scrypto::resource::Bucket(0),
        ]);
        let error = ScryptoValue::from_slice(&buckets).expect_err("Should be an error");
        assert_eq!(
            error,
            ParseScryptoValueError::CustomValueCheckError(
                ScryptoCustomValueCheckError::DuplicateIds
            )
        );
    }
}
