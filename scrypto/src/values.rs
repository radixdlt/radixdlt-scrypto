use sbor::path::{MutableSborPath, SborPath};
use sbor::rust::borrow::Borrow;
use sbor::rust::collections::HashMap;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt;
use sbor::rust::format;
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
    pub component_ids: HashSet<ComponentId>,
    pub refed_component_addresses: HashSet<ComponentAddress>,
    pub resource_addresses: HashSet<ResourceAddress>,
    pub non_fungible_addresses: HashSet<NonFungibleAddress>,
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
            component_ids: checker.components.iter().map(|e| e.0).collect(),
            refed_component_addresses: checker.ref_components,
            resource_addresses: checker.resource_addresses,
            non_fungible_addresses: checker.non_fungible_addresses,
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
            component_ids: HashSet::new(),
            refed_component_addresses: HashSet::new(),
            resource_addresses: HashSet::new(),
            non_fungible_addresses: HashSet::new(),
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
        for component_id in &self.component_ids {
            node_ids.insert(RENodeId::Component(*component_id));
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
        for component_address in &self.component_ids {
            node_ids.insert(RENodeId::Component(*component_address));
        }
        node_ids
    }

    pub fn global_references(&self) -> HashSet<GlobalAddress> {
        let mut node_ids = HashSet::new();
        for component_address in &self.refed_component_addresses {
            node_ids.insert(GlobalAddress::Component(*component_address));
        }
        for resource_address in &self.resource_addresses {
            node_ids.insert(GlobalAddress::Resource(*resource_address));
        }
        for non_fungible_address in &self.non_fungible_addresses {
            node_ids.insert(GlobalAddress::Resource(
                non_fungible_address.resource_address(),
            ));
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
            + self.component_ids.len()
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
    pub non_fungible_addresses: HashSet<NonFungibleAddress>,
}

/// Represents an error when validating a Scrypto-specific value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValueCheckError {
    UnknownTypeId(u8),
    InvalidDecimal(ParseDecimalError),
    InvalidPreciseDecimal(ParsePreciseDecimalError),
    InvalidPackageAddress(AddressError),
    InvalidComponent(ParseComponentError),
    InvalidComponentAddress(AddressError),
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
            non_fungible_addresses: HashSet::new(),
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
                let non_fungible_address = NonFungibleAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidNonFungibleAddress)?;
                self.non_fungible_addresses.insert(non_fungible_address);
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

#[derive(Clone, Copy, Debug)]
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

    pub fn no_manifest_context(bech32_encoder: Option<&'a Bech32Encoder>) -> Self {
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
        ScryptoValueFormatterContext::no_manifest_context(Some(self))
    }
}

impl<'a> Into<ScryptoValueFormatterContext<'a>> for Option<&'a Bech32Encoder> {
    fn into(self) -> ScryptoValueFormatterContext<'a> {
        ScryptoValueFormatterContext::no_manifest_context(self)
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
        ScryptoValueFormatter::format_value(f, &self.dom, context)
    }
}

impl<'a> ContextualDisplay<ScryptoValueFormatterContext<'a>> for Value {
    type Error = ScryptoValueFormatterError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &ScryptoValueFormatterContext<'a>,
    ) -> Result<(), Self::Error> {
        ScryptoValueFormatter::format_value(f, &self, context)
    }
}

impl ScryptoValueFormatter {
    pub fn format_value<F: fmt::Write>(
        f: &mut F,
        value: &Value,
        context: &ScryptoValueFormatterContext,
    ) -> Result<(), ScryptoValueFormatterError> {
        match value {
            // primitive types
            Value::Unit => write!(f, "()")?,
            Value::Bool { value } => write!(f, "{}", value)?,
            Value::I8 { value } => write!(f, "{}i8", value)?,
            Value::I16 { value } => write!(f, "{}i16", value)?,
            Value::I32 { value } => write!(f, "{}i32", value)?,
            Value::I64 { value } => write!(f, "{}i64", value)?,
            Value::I128 { value } => write!(f, "{}i128", value)?,
            Value::U8 { value } => write!(f, "{}u8", value)?,
            Value::U16 { value } => write!(f, "{}u16", value)?,
            Value::U32 { value } => write!(f, "{}u32", value)?,
            Value::U64 { value } => write!(f, "{}u64", value)?,
            Value::U128 { value } => write!(f, "{}u128", value)?,
            Value::String { value } => write!(f, "\"{}\"", value)?,
            // struct & enum
            Value::Struct { fields } => {
                f.write_str("Struct(")?;
                Self::format_elements(f, fields, context)?;
                f.write_str(")")?;
            }
            Value::Enum { name, fields } => {
                f.write_str("Enum(\"")?;
                f.write_str(name)?;
                f.write_str("\"")?;
                if !fields.is_empty() {
                    f.write_str(", ")?;
                    Self::format_elements(f, fields, context)?;
                }
                f.write_str(")")?;
            }
            // rust types
            Value::Option { value } => match value.borrow() {
                Some(x) => {
                    f.write_str("Some(")?;
                    Self::format_value(f, x, context)?;
                    f.write_str(")")?;
                }
                None => write!(f, "None")?,
            },
            Value::Array {
                element_type_id,
                elements,
            } => {
                f.write_str("Array<")?;
                Self::format_type_id(f, *element_type_id)?;
                f.write_str(">(")?;
                Self::format_elements(f, elements, context)?;
                f.write_str(")")?;
            }
            Value::Tuple { elements } => {
                f.write_str("Tuple(")?;
                Self::format_elements(f, elements, context)?;
                f.write_str(")")?;
            }
            Value::Result { value } => match value.borrow() {
                Ok(x) => {
                    f.write_str("Ok(")?;
                    Self::format_value(f, x, context)?;
                    f.write_str(")")?
                }
                Err(x) => {
                    f.write_str("Err(")?;
                    Self::format_value(f, x, context)?;
                    f.write_str(")")?;
                }
            },
            // collections
            Value::List {
                element_type_id,
                elements,
            } => {
                f.write_str("Vec<")?;
                Self::format_type_id(f, *element_type_id)?;
                f.write_str(">(")?;
                Self::format_elements(f, elements, context)?;
                f.write_str(")")?;
            }
            Value::Set {
                element_type_id,
                elements,
            } => {
                f.write_str("Set<")?;
                Self::format_type_id(f, *element_type_id)?;
                f.write_str(">(")?;
                Self::format_elements(f, elements, context)?;
                f.write_str(")")?;
            }
            Value::Map {
                key_type_id,
                value_type_id,
                elements,
            } => {
                f.write_str("Map<")?;
                Self::format_type_id(f, *key_type_id)?;
                f.write_str(", ")?;
                Self::format_type_id(f, *value_type_id)?;
                f.write_str(">(")?;
                Self::format_elements(f, elements, context)?;
                f.write_str(")")?;
            }
            // custom types
            Value::Custom { type_id, bytes } => {
                Self::format_custom_value(f, *type_id, bytes, context)?;
            }
        };
        Ok(())
    }

    pub fn format_type_id<F: fmt::Write>(
        f: &mut F,
        type_id: u8,
    ) -> Result<(), ScryptoValueFormatterError> {
        if let Some(ty) = ScryptoType::from_id(type_id) {
            write!(f, "{}", ty.name())?;
            return Ok(());
        }

        match type_id {
            // primitive types
            TYPE_UNIT => f.write_str("Unit")?,
            TYPE_BOOL => f.write_str("Bool")?,
            TYPE_I8 => f.write_str("I8")?,
            TYPE_I16 => f.write_str("I16")?,
            TYPE_I32 => f.write_str("I32")?,
            TYPE_I64 => f.write_str("I64")?,
            TYPE_I128 => f.write_str("I128")?,
            TYPE_U8 => f.write_str("U8")?,
            TYPE_U16 => f.write_str("U16")?,
            TYPE_U32 => f.write_str("U32")?,
            TYPE_U64 => f.write_str("U64")?,
            TYPE_U128 => f.write_str("U128")?,
            TYPE_STRING => f.write_str("String")?,
            // struct & enum
            TYPE_STRUCT => f.write_str("Struct")?,
            TYPE_ENUM => f.write_str("Enum")?,
            TYPE_OPTION => f.write_str("Option")?,
            TYPE_RESULT => f.write_str("Result")?,
            // composite
            TYPE_ARRAY => f.write_str("Array")?,
            TYPE_TUPLE => f.write_str("Tuple")?,
            // collections
            TYPE_LIST => f.write_str("List")?,
            TYPE_SET => f.write_str("Set")?,
            TYPE_MAP => f.write_str("Map")?,
            //
            _ => Err(ScryptoValueFormatterError::InvalidTypeId(type_id))?,
        };

        Ok(())
    }

    pub fn format_elements<F: fmt::Write>(
        f: &mut F,
        values: &[Value],
        context: &ScryptoValueFormatterContext,
    ) -> Result<(), ScryptoValueFormatterError> {
        for (i, x) in values.iter().enumerate() {
            if i != 0 {
                f.write_str(", ")?;
            }
            Self::format_value(f, x, context)?;
        }
        Ok(())
    }

    pub fn format_custom_value<F: fmt::Write>(
        f: &mut F,
        type_id: u8,
        data: &[u8],
        context: &ScryptoValueFormatterContext,
    ) -> Result<(), ScryptoValueFormatterError> {
        let scrypto_type = ScryptoType::from_id(type_id);
        if scrypto_type.is_none() {
            Err(ScryptoCustomValueCheckError::UnknownTypeId(type_id))?;
        }
        match scrypto_type.unwrap() {
            ScryptoType::Decimal => {
                let value = Decimal::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidDecimal)?;
                write!(f, "Decimal(\"{}\")", value)?;
            }
            ScryptoType::PreciseDecimal => {
                let value = PreciseDecimal::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidPreciseDecimal)?;
                write!(f, "PreciseDecimal(\"{}\")", value)?;
            }
            ScryptoType::PackageAddress => {
                let value = PackageAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidPackageAddress)?;
                f.write_str("PackageAddress(\"")?;
                value
                    .format(f, context.bech32_encoder)
                    .map_err(ScryptoCustomValueCheckError::InvalidPackageAddress)?;
                f.write_str("\")")?;
            }
            ScryptoType::ComponentAddress => {
                let value = ComponentAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidComponentAddress)?;
                f.write_str("ComponentAddress(\"")?;
                value
                    .format(f, context.bech32_encoder)
                    .map_err(ScryptoCustomValueCheckError::InvalidComponentAddress)?;
                f.write_str("\")")?;
            }
            ScryptoType::Component => {
                let value = Component::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidComponent)?;
                write!(f, "Component(\"{}\")", value)?;
            }
            ScryptoType::KeyValueStore => {
                let value = KeyValueStore::<(), ()>::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidKeyValueStore)?;
                write!(f, "KeyValueStore(\"{}\")", value)?;
            }
            ScryptoType::Hash => {
                let value =
                    Hash::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidHash)?;
                write!(f, "Hash(\"{}\")", value)?;
            }
            ScryptoType::EcdsaSecp256k1PublicKey => {
                let value = EcdsaSecp256k1PublicKey::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidEcdsaSecp256k1PublicKey)?;
                write!(f, "EcdsaSecp256k1PublicKey(\"{}\")", value)?;
            }
            ScryptoType::EcdsaSecp256k1Signature => {
                let value = EcdsaSecp256k1Signature::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidEcdsaSecp256k1Signature)?;
                write!(f, "EcdsaSecp256k1Signature(\"{}\")", value)?;
            }
            ScryptoType::EddsaEd25519PublicKey => {
                let value = EddsaEd25519PublicKey::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidEddsaEd25519PublicKey)?;
                write!(f, "EddsaEd25519PublicKey(\"{}\")", value)?;
            }
            ScryptoType::EddsaEd25519Signature => {
                let value = EddsaEd25519Signature::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidEddsaEd25519Signature)?;
                write!(f, "EddsaEd25519Signature(\"{}\")", value)?;
            }
            ScryptoType::Bucket => {
                let value =
                    Bucket::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidBucket)?;
                if let Some(name) = context.get_bucket_name(&value.0) {
                    write!(f, "Bucket(\"{}\")", name)?;
                } else {
                    write!(f, "Bucket({}u32)", value.0)?;
                }
            }
            ScryptoType::Proof => {
                let value =
                    Proof::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidProof)?;
                if let Some(name) = context.get_proof_name(&value.0) {
                    write!(f, "Proof(\"{}\")", name)?;
                } else {
                    write!(f, "Proof({}u32)", value.0)?;
                }
            }
            ScryptoType::Vault => {
                let value =
                    Vault::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidVault)?;
                write!(f, "Vault(\"{}\")", value)?;
            }
            ScryptoType::NonFungibleId => {
                let value = NonFungibleId::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidNonFungibleId)?;
                write!(f, "NonFungibleId(\"{}\")", value)?;
            }
            ScryptoType::NonFungibleAddress => {
                let value = NonFungibleAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidNonFungibleAddress)?;
                write!(f, "NonFungibleAddress(\"{}\")", value)?;
            }
            ScryptoType::ResourceAddress => {
                let value = ResourceAddress::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidResourceAddress)?;
                f.write_str("ResourceAddress(\"")?;
                value
                    .format(f, context.bech32_encoder)
                    .map_err(ScryptoCustomValueCheckError::InvalidResourceAddress)?;
                f.write_str("\")")?;
            }
            ScryptoType::Expression => {
                let value = Expression::try_from(data)
                    .map_err(ScryptoCustomValueCheckError::InvalidExpression)?;
                write!(f, "Expression(\"{}\")", value)?;
            }
            ScryptoType::Blob => {
                let value =
                    Blob::try_from(data).map_err(ScryptoCustomValueCheckError::InvalidBlob)?;
                write!(f, "Blob(\"{}\")", value)?;
            }
        }
        Ok(())
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
