use sbor::path::{MutableSborPath, SborPath};
use sbor::rust::borrow::Borrow;
use sbor::rust::collections::HashMap;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::string::*;
use sbor::rust::vec::Vec;
use sbor::type_id::*;
use sbor::{any::*, *};

use crate::abi::*;
use crate::address::{AddressError, Bech32Encoder};
use crate::buffer::*;
use crate::component::*;
use crate::core::*;
use crate::crypto::*;
use crate::engine::types::*;
use crate::math::*;
use crate::misc::*;
use crate::resource::*;

pub enum ScryptoValueReplaceError {
    ProofIdNotFound(ProofId),
    BucketIdNotFound(BucketId),
}

/// A Scrypto value is a SBOR value of which the custom types are the ones defined by `ScryptoType`.
#[derive(Clone, PartialEq, Eq)]
pub struct ScryptoValue {
    pub raw: Vec<u8>,
    pub dom: Value,
    pub expressions: Vec<(Expression, SborPath)>,
    pub bucket_ids: HashMap<BucketId, SborPath>,
    pub proof_ids: HashMap<ProofId, SborPath>,
    pub vault_ids: HashSet<VaultId>,
    pub kv_store_ids: HashSet<KeyValueStoreId>,
    pub owned_component_addresses: HashSet<ComponentAddress>,
    pub refed_component_addresses: HashSet<ComponentAddress>,
    pub resource_addresses: HashSet<ResourceAddress>,
}

impl ScryptoValue {
    pub fn unit() -> Self {
        Self::from_typed(&())
    }

    pub fn from_typed<T: Encode>(value: &T) -> Self {
        let bytes = scrypto_encode(value);
        Self::from_slice(&bytes).expect("Failed to convert trusted value into ScryptoValue")
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, DecodeError> {
        let value = decode_any(slice)?;
        Self::from_value(value)
    }

    pub fn from_value(value: Value) -> Result<Self, DecodeError> {
        let mut checker = ScryptoCustomValueChecker::new();
        traverse_any(&mut MutableSborPath::new(), &value, &mut checker)
            .map_err(|e| DecodeError::CustomError(format!("{:?}", e)))?;

        Ok(Self {
            raw: encode_any(&value),
            dom: value,
            expressions: checker.expressions,
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
            kv_store_ids: checker.kv_stores,
            owned_component_addresses: checker.components.iter().map(|e| e.0).collect(),
            refed_component_addresses: checker.ref_components,
            resource_addresses: checker.resource_addresses,
        })
    }

    pub fn from_slice_no_custom_values(slice: &[u8]) -> Result<Self, DecodeError> {
        let value = decode_any(slice)?;
        let mut checker = ScryptoNoCustomValuesChecker {};
        traverse_any(&mut MutableSborPath::new(), &value, &mut checker)
            .map_err(|e| DecodeError::CustomError(format!("{:?}", e)))?;
        Ok(Self {
            raw: encode_any(&value),
            dom: value,
            expressions: Vec::new(),
            bucket_ids: HashMap::new(),
            proof_ids: HashMap::new(),
            vault_ids: HashSet::new(),
            kv_store_ids: HashSet::new(),
            owned_component_addresses: HashSet::new(),
            refed_component_addresses: HashSet::new(),
            resource_addresses: HashSet::new(),
        })
    }

    pub fn node_ids(&self) -> HashSet<RENodeId> {
        let mut node_ids = HashSet::new();
        for vault_id in &self.vault_ids {
            node_ids.insert(RENodeId::Vault(*vault_id));
        }
        for kv_store_id in &self.kv_store_ids {
            node_ids.insert(RENodeId::KeyValueStore(*kv_store_id));
        }
        for component_address in &self.owned_component_addresses {
            node_ids.insert(RENodeId::Component(*component_address));
        }
        for (bucket_id, _) in &self.bucket_ids {
            node_ids.insert(RENodeId::Bucket(*bucket_id));
        }
        for (proof_id, _) in &self.proof_ids {
            node_ids.insert(RENodeId::Proof(*proof_id));
        }
        node_ids
    }

    pub fn stored_node_ids(&self) -> HashSet<RENodeId> {
        let mut node_ids = HashSet::new();
        for vault_id in &self.vault_ids {
            node_ids.insert(RENodeId::Vault(*vault_id));
        }
        for kv_store_id in &self.kv_store_ids {
            node_ids.insert(RENodeId::KeyValueStore(*kv_store_id));
        }
        for component_address in &self.owned_component_addresses {
            node_ids.insert(RENodeId::Component(*component_address));
        }
        node_ids
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

    pub fn value_count(&self) -> usize {
        self.bucket_ids.len()
            + self.proof_ids.len()
            + self.vault_ids.len()
            + self.owned_component_addresses.len()
    }
}

impl fmt::Debug for ScryptoValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.display(ScryptoValueFormatterContext::no_context())
        )
    }
}

/// Represents an error when validating a Scrypto-specific value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoNoCustomValuesCheckError {
    CustomValueNotAllowed(u8),
}

/// A checker the check a Scrypto-specific value.
struct ScryptoNoCustomValuesChecker {}

impl CustomValueVisitor for ScryptoNoCustomValuesChecker {
    type Err = ScryptoNoCustomValuesCheckError;

    fn visit(
        &mut self,
        _path: &mut MutableSborPath,
        type_id: u8,
        _data: &[u8],
    ) -> Result<(), Self::Err> {
        return Err(ScryptoNoCustomValuesCheckError::CustomValueNotAllowed(
            type_id,
        ));
    }
}

/// A checker the check a Scrypto-specific value.
pub struct ScryptoCustomValueChecker {
    pub expressions: Vec<(Expression, SborPath)>,
    pub buckets: HashMap<Bucket, SborPath>,
    pub proofs: HashMap<Proof, SborPath>,
    pub vaults: HashSet<Vault>,
    pub kv_stores: HashSet<KeyValueStoreId>,
    pub components: HashSet<Component>,
    pub ref_components: HashSet<ComponentAddress>,
    pub resource_addresses: HashSet<ResourceAddress>,
}

/// Represents an error when validating a Scrypto-specific value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValueCheckError {
    UnknownTypeId(u8),
    InvalidDecimal(ParseDecimalError),
    InvalidPreciseDecimal(ParsePreciseDecimalError),
    InvalidPackageAddress(AddressError),
    InvalidComponentAddress(AddressError),
    InvalidComponent(AddressError),
    InvalidResourceAddress(AddressError),
    InvalidHash(ParseHashError),
    InvalidEcdsaSecp256k1PublicKey(ParseEcdsaSecp256k1PublicKeyError),
    InvalidEcdsaSecp256k1Signature(ParseEcdsaSecp256k1SignatureError),
    InvalidEddsaEd25519PublicKey(ParseEddsaEd25519PublicKeyError),
    InvalidEddsaEd25519Signature(ParseEddsaEd25519SignatureError),
    InvalidBucket(ParseBucketError),
    InvalidProof(ParseProofError),
    InvalidKeyValueStore(ParseKeyValueStoreError),
    InvalidVault(ParseVaultError),
    InvalidNonFungibleId(ParseNonFungibleIdError),
    InvalidNonFungibleAddress(ParseNonFungibleAddressError),
    InvalidExpression(ParseExpressionError),
    InvalidBlob(ParseBlobError),
    DuplicateIds,
}

impl ScryptoCustomValueChecker {
    pub fn new() -> Self {
        Self {
            expressions: Vec::new(),
            buckets: HashMap::new(),
            proofs: HashMap::new(),
            vaults: HashSet::new(),
            kv_stores: HashSet::new(),
            components: HashSet::new(),
            ref_components: HashSet::new(),
            resource_addresses: HashSet::new(),
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
        match ScryptoType::from_id(type_id).ok_or(Self::Err::UnknownTypeId(type_id))? {
            ScryptoType::PackageAddress => {
                PackageAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidPackageAddress)?;
            }
            ScryptoType::ComponentAddress => {
                let component_address = ComponentAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidComponentAddress)?;
                self.ref_components.insert(component_address);
            }
            ScryptoType::Component => {
                let component = Component::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidComponent)?;
                if !self.components.insert(component) {
                    return Err(ScryptoCustomValueCheckError::DuplicateIds);
                }
            }
            ScryptoType::KeyValueStore => {
                let kv_store_id: KeyValueStoreId = match data.len() {
                    36 => (
                        Hash(copy_u8_array(&data[0..32])),
                        u32::from_le_bytes(copy_u8_array(&data[32..])),
                    ),
                    _ => {
                        return Err(ScryptoCustomValueCheckError::InvalidKeyValueStore(
                            ParseKeyValueStoreError::InvalidLength(data.len()),
                        ))
                    }
                };

                if !self.kv_stores.insert(kv_store_id) {
                    return Err(ScryptoCustomValueCheckError::DuplicateIds);
                }
            }
            ScryptoType::Hash => {
                Hash::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidHash)?;
            }
            ScryptoType::EcdsaSecp256k1PublicKey => {
                EcdsaSecp256k1PublicKey::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidEcdsaSecp256k1PublicKey)?;
            }
            ScryptoType::EcdsaSecp256k1Signature => {
                EcdsaSecp256k1Signature::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidEcdsaSecp256k1Signature)?;
            }
            ScryptoType::EddsaEd25519PublicKey => {
                EddsaEd25519PublicKey::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidEddsaEd25519PublicKey)?;
            }
            ScryptoType::EddsaEd25519Signature => {
                EddsaEd25519Signature::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidEddsaEd25519Signature)?;
            }
            ScryptoType::Decimal => {
                Decimal::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidDecimal)?;
            }
            ScryptoType::PreciseDecimal => {
                PreciseDecimal::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidPreciseDecimal)?;
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
                let resource_address = ResourceAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidResourceAddress)?;
                self.resource_addresses.insert(resource_address);
            }
            ScryptoType::Expression => {
                let expression = Expression::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidExpression)?;
                self.expressions.push((expression, path.clone().into()));
            }
            ScryptoType::Blob => {
                Blob::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidBlob)?;
            }
        }
        Ok(())
    }
}

/// Utility that formats any Scrypto value.
pub struct ScryptoValueFormatter {}

pub struct ScryptoValueFormatterContext<'a> {
    bech32_encoder: Option<&'a Bech32Encoder>,
    bucket_names: Option<&'a HashMap<BucketId, String>>,
    proof_names: Option<&'a HashMap<ProofId, String>>,
}

impl<'a> ScryptoValueFormatterContext<'a> {
    pub fn no_context() -> Self {
        Self {
            bech32_encoder: None,
            bucket_names: None,
            proof_names: None,
        }
    }

    pub fn new_with_no_manifest_context(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
        Self {
            bech32_encoder,
            bucket_names: None,
            proof_names: None,
        }
    }

    pub fn with_manifest_context(
        bech32_encoder: Option<&'a Bech32Encoder>,
        bucket_names: &'a HashMap<BucketId, String>,
        proof_names: &'a HashMap<ProofId, String>,
    ) -> Self {
        Self {
            bech32_encoder,
            bucket_names: Some(bucket_names),
            proof_names: Some(proof_names),
        }
    }

    pub fn get_bucket_name(&self, bucket_id: &BucketId) -> Option<&str> {
        self.bucket_names
            .and_then(|names| names.get(bucket_id).map(|s| s.as_str()))
    }

    pub fn get_proof_name(&self, proof_id: &ProofId) -> Option<&str> {
        self.proof_names
            .and_then(|names| names.get(proof_id).map(|s| s.as_str()))
    }
}

impl<'a> Into<ScryptoValueFormatterContext<'a>> for &'a Bech32Encoder {
    fn into(self) -> ScryptoValueFormatterContext<'a> {
        ScryptoValueFormatterContext::new_with_no_manifest_context(Some(self))
    }
}

impl<'a> Into<ScryptoValueFormatterContext<'a>> for Option<&'a Bech32Encoder> {
    fn into(self) -> ScryptoValueFormatterContext<'a> {
        ScryptoValueFormatterContext::new_with_no_manifest_context(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoValueFormatterError {
    InvalidTypeId(u8),
    InvalidCustomData(ScryptoCustomValueCheckError),
    FormatError(fmt::Error),
}

impl From<ScryptoCustomValueCheckError> for ScryptoValueFormatterError {
    fn from(error: ScryptoCustomValueCheckError) -> Self {
        ScryptoValueFormatterError::InvalidCustomData(error)
    }
}

impl From<fmt::Error> for ScryptoValueFormatterError {
    fn from(error: fmt::Error) -> Self {
        ScryptoValueFormatterError::FormatError(error)
    }
}

impl<'a> ContextualDisplay<ScryptoValueFormatterContext<'a>> for ScryptoValue {
    type Error = ScryptoValueFormatterError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueFormatterContext<'a>,
    ) -> Result<(), Self::Error> {
        write!(
            f,
            "{}",
            ScryptoValueFormatter::format_value(&self.dom, context)?
        )?;
        Ok(())
    }
}

impl ScryptoValueFormatter {
    pub fn format_value(
        value: &Value,
        context: &ScryptoValueFormatterContext,
    ) -> Result<String, ScryptoValueFormatterError> {
        Ok(match value {
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
                format!("Struct({})", Self::format_elements(fields, context)?)
            }
            Value::Enum { name, fields } => {
                format!(
                    "Enum(\"{}\"{}{})",
                    name,
                    if fields.is_empty() { "" } else { ", " },
                    Self::format_elements(fields, context)?
                )
            }
            // rust types
            Value::Option { value } => match value.borrow() {
                Some(x) => format!("Some({})", Self::format_value(x, context)?),
                None => "None".to_string(),
            },
            Value::Array {
                element_type_id,
                elements,
            } => format!(
                "Array<{}>({})",
                Self::format_type_id(*element_type_id)?,
                Self::format_elements(elements, context)?
            ),
            Value::Tuple { elements } => {
                format!("Tuple({})", Self::format_elements(elements, context)?)
            }
            Value::Result { value } => match value.borrow() {
                Ok(x) => format!("Ok({})", Self::format_value(x, context)?),
                Err(x) => format!("Err({})", Self::format_value(x, context)?),
            },
            // collections
            Value::List {
                element_type_id,
                elements,
            } => {
                format!(
                    "Vec<{}>({})",
                    Self::format_type_id(*element_type_id)?,
                    Self::format_elements(elements, context)?
                )
            }
            Value::Set {
                element_type_id,
                elements,
            } => format!(
                "Set<{}>({})",
                Self::format_type_id(*element_type_id)?,
                Self::format_elements(elements, context)?
            ),
            Value::Map {
                key_type_id,
                value_type_id,
                elements,
            } => format!(
                "Map<{}, {}>({})",
                Self::format_type_id(*key_type_id)?,
                Self::format_type_id(*value_type_id)?,
                Self::format_elements(elements, context)?
            ),
            // custom types
            Value::Custom { type_id, bytes } => {
                Self::format_custom_value(*type_id, bytes, context)?
            }
        })
    }

    pub fn format_type_id(type_id: u8) -> Result<String, ScryptoValueFormatterError> {
        if let Some(ty) = ScryptoType::from_id(type_id) {
            return Ok(ty.name());
        }

        Ok(match type_id {
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
            TYPE_RESULT => "Result",
            // composite
            TYPE_ARRAY => "Array",
            TYPE_TUPLE => "Tuple",
            // collections
            TYPE_LIST => "List",
            TYPE_SET => "Set",
            TYPE_MAP => "Map",
            //
            _ => Err(ScryptoValueFormatterError::InvalidTypeId(type_id))?,
        }
        .to_string())
    }

    pub fn format_elements(
        values: &[Value],
        context: &ScryptoValueFormatterContext,
    ) -> Result<String, ScryptoValueFormatterError> {
        let mut buf = String::new();
        for (i, x) in values.iter().enumerate() {
            if i != 0 {
                buf.push_str(", ");
            }
            buf.push_str(Self::format_value(x, context)?.as_str());
        }
        Ok(buf)
    }

    pub fn format_custom_value(
        type_id: u8,
        data: &[u8],
        context: &ScryptoValueFormatterContext,
    ) -> Result<String, ScryptoValueFormatterError> {
        let scrypto_type = ScryptoType::from_id(type_id);
        if scrypto_type.is_none() {
            Err(ScryptoCustomValueCheckError::UnknownTypeId(type_id))?;
        }
        Ok(match scrypto_type.unwrap() {
            ScryptoType::Decimal => Decimal::try_from(data)
                .map(|d| format!("Decimal(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidDecimal)?,
            ScryptoType::PreciseDecimal => PreciseDecimal::try_from(data)
                .map(|d| format!("PreciseDecimal(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidPreciseDecimal)?,
            ScryptoType::PackageAddress => PackageAddress::try_from(data)
                .map(|address| {
                    format!(
                        "PackageAddress(\"{}\")",
                        address.display(context.bech32_encoder)
                    )
                })
                .map_err(ScryptoCustomValueCheckError::InvalidPackageAddress)?,
            ScryptoType::ComponentAddress => ComponentAddress::try_from(data)
                .map(|address| {
                    format!(
                        "ComponentAddress(\"{}\")",
                        address.display(context.bech32_encoder)
                    )
                })
                .map_err(ScryptoCustomValueCheckError::InvalidComponentAddress)?,
            ScryptoType::Component => Component::try_from(data)
                .map(|d| format!("Component(\"{}\")", d.0.display(context.bech32_encoder)))
                .map_err(ScryptoCustomValueCheckError::InvalidComponent)?,
            ScryptoType::KeyValueStore => KeyValueStore::<(), ()>::try_from(data)
                .map(|d| format!("KeyValueStore(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidKeyValueStore)?,
            ScryptoType::Hash => Hash::try_from(data)
                .map(|d| format!("Hash(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidHash)?,
            ScryptoType::EcdsaSecp256k1PublicKey => EcdsaSecp256k1PublicKey::try_from(data)
                .map(|d| format!("EcdsaSecp256k1PublicKey(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidEcdsaSecp256k1PublicKey)?,
            ScryptoType::EcdsaSecp256k1Signature => EcdsaSecp256k1Signature::try_from(data)
                .map(|d| format!("EcdsaSecp256k1Signature(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidEcdsaSecp256k1Signature)?,
            ScryptoType::EddsaEd25519PublicKey => EddsaEd25519PublicKey::try_from(data)
                .map(|d| format!("EddsaEd25519PublicKey(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidEddsaEd25519PublicKey)?,
            ScryptoType::EddsaEd25519Signature => EddsaEd25519Signature::try_from(data)
                .map(|d| format!("EddsaEd25519Signature(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidEddsaEd25519Signature)?,
            ScryptoType::Bucket => Bucket::try_from(data)
                .map(|bucket| {
                    if let Some(name) = context.get_bucket_name(&bucket.0) {
                        format!("Bucket(\"{}\")", name)
                    } else {
                        format!("Bucket({}u32)", bucket.0)
                    }
                })
                .map_err(ScryptoCustomValueCheckError::InvalidBucket)?,
            ScryptoType::Proof => Proof::try_from(data)
                .map(|proof| {
                    if let Some(name) = context.get_proof_name(&proof.0) {
                        format!("Proof(\"{}\")", name)
                    } else {
                        format!("Proof({}u32)", proof.0)
                    }
                })
                .map_err(ScryptoCustomValueCheckError::InvalidProof)?,
            ScryptoType::Vault => Vault::try_from(data)
                .map(|d| format!("Vault(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidVault)?,
            ScryptoType::NonFungibleId => NonFungibleId::try_from(data)
                .map(|d| format!("NonFungibleId(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidNonFungibleId)?,
            ScryptoType::NonFungibleAddress => NonFungibleAddress::try_from(data)
                .map(|d| format!("NonFungibleAddress(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidNonFungibleAddress)?,
            ScryptoType::ResourceAddress => ResourceAddress::try_from(data)
                .map(|address| {
                    format!(
                        "ResourceAddress(\"{}\")",
                        address.display(context.bech32_encoder)
                    )
                })
                .map_err(ScryptoCustomValueCheckError::InvalidResourceAddress)?,
            ScryptoType::Expression => Expression::try_from(data)
                .map(|d| format!("Expression(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidExpression)?,
            ScryptoType::Blob => Blob::try_from(data)
                .map(|d| format!("Blob(\"{}\")", d))
                .map_err(ScryptoCustomValueCheckError::InvalidBlob)?,
        })
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
        assert_eq!(error, DecodeError::CustomError("DuplicateIds".to_string()));
    }
}
