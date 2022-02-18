use sbor::{any::*, *};

use crate::component::*;
use crate::crypto::*;
use crate::engine::types::BucketId;
use crate::engine::types::ProofId;
use crate::math::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::format;
use crate::rust::string::String;
use crate::rust::vec::Vec;

macro_rules! custom_type {
    ($t:ty, $ct:expr, $generics: expr) => {
        impl TypeId for $t {
            #[inline]
            fn type_id() -> u8 {
                $ct.id()
            }
        }

        impl Encode for $t {
            fn encode_value(&self, encoder: &mut Encoder) {
                let bytes = self.to_vec();
                encoder.write_len(bytes.len());
                encoder.write_slice(&bytes);
            }
        }

        impl Decode for $t {
            fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
                let len = decoder.read_len()?;
                let slice = decoder.read_bytes(len)?;
                Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData($ct.id()))
            }
        }

        impl Describe for $t {
            fn describe() -> Type {
                Type::Custom {
                    name: $ct.name(),
                    generics: $generics,
                }
            }
        }
    };
}

pub(crate) use custom_type;

/// Scrypto types that are encoded as custom SBOR types.
///
/// Any encode-able type in Scrypto library that requires special interpretation
/// must be declared as a custom type.
///
/// Custom types must be encoded as `[length + bytes]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomType {
    // component
    PackageId,
    ComponentId,
    LazyMap,

    // crypto
    Hash,

    // math
    Decimal,
    BigDecimal,

    // resource,
    Bucket,
    Proof,
    Vault,
    NonFungibleKey,
    ResourceDefId,
}

const MAPPING: [(CustomType, u8, &str); 11] = [
    (CustomType::PackageId, 0x80, "PackageId"),
    (CustomType::ComponentId, 0x81, "ComponentId"),
    (CustomType::LazyMap, 0x82, "LazyMap"),
    (CustomType::Hash, 0x90, "Hash"),
    (CustomType::Decimal, 0xa1, "Decimal"),
    (CustomType::BigDecimal, 0xa2, "BigDecimal"),
    (CustomType::Bucket, 0xb1, "Bucket"),
    (CustomType::Proof, 0xb2, "Proof"),
    (CustomType::Vault, 0xb3, "Vault"),
    (CustomType::NonFungibleKey, 0xb4, "NonFungibleKey"),
    (CustomType::ResourceDefId, 0xb5, "ResourceDefId"),
];

impl CustomType {
    // TODO: optimize to get rid of loops

    pub fn from_id(id: u8) -> Option<CustomType> {
        MAPPING.iter().filter(|e| e.1 == id).map(|e| e.0).next()
    }

    pub fn from_name(name: &str) -> Option<CustomType> {
        MAPPING.iter().filter(|e| e.2 == name).map(|e| e.0).next()
    }

    pub fn id(&self) -> u8 {
        MAPPING
            .iter()
            .filter(|e| e.0 == *self)
            .map(|e| e.1)
            .next()
            .unwrap()
    }

    pub fn name(&self) -> String {
        MAPPING
            .iter()
            .filter(|e| e.0 == *self)
            .map(|e| e.2)
            .next()
            .unwrap()
            .to_owned()
    }
}

pub struct CustomValueValidator {
    pub buckets: Vec<Bucket>,
    pub proofs: Vec<Proof>,
    pub vaults: Vec<Vault>,
    pub lazy_maps: Vec<LazyMap<(), ()>>,
}

#[derive(Debug, Clone)]
pub enum CustomValueValidatorError {
    DecodeError(DecodeError),
    InvalidTypeId(u8),
    InvalidDecimal(ParseDecimalError),
    InvalidBigDecimal(ParseBigDecimalError),
    InvalidPackageId(ParsePackageIdError),
    InvalidComponentId(ParseComponentIdError),
    InvalidResourceDefId(ParseResourceDefIdError),
    InvalidHash(ParseHashError),
    InvalidBucket(ParseBucketError),
    InvalidProof(ParseProofError),
    InvalidLazyMap(ParseLazyMapError),
    InvalidVault(ParseVaultError),
    InvalidNonFungibleKey(ParseNonFungibleKeyError),
}

impl CustomValueValidator {
    pub fn new() -> Self {
        Self {
            buckets: Vec::new(),
            proofs: Vec::new(),
            vaults: Vec::new(),
            lazy_maps: Vec::new(),
        }
    }
}

impl CustomValueVisitor for CustomValueValidator {
    type Err = CustomValueValidatorError;

    fn visit(&mut self, type_id: u8, data: &[u8]) -> Result<(), Self::Err> {
        match CustomType::from_id(type_id).ok_or(Self::Err::InvalidTypeId(type_id))? {
            CustomType::PackageId => {
                PackageId::try_from(data).map_err(CustomValueValidatorError::InvalidPackageId)?;
            }
            CustomType::ComponentId => {
                ComponentId::try_from(data)
                    .map_err(CustomValueValidatorError::InvalidComponentId)?;
            }
            CustomType::LazyMap => {
                self.lazy_maps.push(
                    LazyMap::try_from(data).map_err(CustomValueValidatorError::InvalidLazyMap)?,
                );
            }
            CustomType::Hash => {
                Hash::try_from(data).map_err(CustomValueValidatorError::InvalidHash)?;
            }
            CustomType::Decimal => {
                Decimal::try_from(data).map_err(CustomValueValidatorError::InvalidDecimal)?;
            }
            CustomType::BigDecimal => {
                BigDecimal::try_from(data).map_err(CustomValueValidatorError::InvalidBigDecimal)?;
            }
            CustomType::Bucket => {
                self.buckets.push(
                    Bucket::try_from(data).map_err(CustomValueValidatorError::InvalidBucket)?,
                );
            }
            CustomType::Proof => {
                self.proofs
                    .push(Proof::try_from(data).map_err(CustomValueValidatorError::InvalidProof)?);
            }
            CustomType::Vault => {
                self.vaults
                    .push(Vault::try_from(data).map_err(CustomValueValidatorError::InvalidVault)?);
            }
            CustomType::NonFungibleKey => {
                NonFungibleKey::try_from(data)
                    .map_err(CustomValueValidatorError::InvalidNonFungibleKey)?;
            }
            CustomType::ResourceDefId => {
                ResourceDefId::try_from(data)
                    .map_err(CustomValueValidatorError::InvalidResourceDefId)?;
            }
        }
        Ok(())
    }
}

pub struct CustomValueFormatter {}

impl CustomValueFormatter {
    /// Format a custom value (checked) using the notation introduced by Transaction Manifest.
    ///
    /// # Panics
    /// If the input data or type id is invalid
    pub fn format(
        type_id: u8,
        data: &[u8],
        bucket_ids: &HashMap<BucketId, String>,
        proof_ids: &HashMap<ProofId, String>,
    ) -> String {
        match CustomType::from_id(type_id).unwrap() {
            CustomType::Decimal => format!("Decimal(\"{}\")", Decimal::try_from(data).unwrap()),
            CustomType::BigDecimal => {
                format!("BigDecimal(\"{}\")", BigDecimal::try_from(data).unwrap())
            }
            CustomType::PackageId => {
                format!("PackageId(\"{}\")", PackageId::try_from(data).unwrap())
            }
            CustomType::ComponentId => {
                format!("ComponentId(\"{}\")", ComponentId::try_from(data).unwrap())
            }
            CustomType::LazyMap => format!(
                "LazyMap(\"{}\")",
                LazyMap::<(), ()>::try_from(data).unwrap()
            ),
            CustomType::Hash => format!("Hash(\"{}\")", Hash::try_from(data).unwrap()),
            CustomType::Bucket => {
                let bucket = Bucket::try_from(data).unwrap();
                if let Some(name) = bucket_ids.get(&bucket.0) {
                    format!("Bucket(\"{}\")", name)
                } else {
                    format!("Bucket({}u32)", bucket.0)
                }
            }
            CustomType::Proof => {
                let proof = Proof::try_from(data).unwrap();
                if let Some(name) = proof_ids.get(&proof.0) {
                    format!("Proof(\"{}\")", name)
                } else {
                    format!("Proof({}u32)", proof.0)
                }
            }
            CustomType::Vault => format!("Vault(\"{}\")", Vault::try_from(data).unwrap()),
            CustomType::NonFungibleKey => format!(
                "NonFungibleKey(\"{}\")",
                NonFungibleKey::try_from(data).unwrap()
            ),
            CustomType::ResourceDefId => format!(
                "ResourceDefId(\"{}\")",
                ResourceDefId::try_from(data).unwrap()
            ),
        }
    }
}
