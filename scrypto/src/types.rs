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

/// A macro to help create a Scrypto-specific type.
macro_rules! scrypto_type {
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
            fn describe() -> sbor::describe::Type {
                sbor::describe::Type::Custom {
                    name: $ct.name(),
                    generics: $generics,
                }
            }
        }
    };
}

pub(crate) use scrypto_type;

/// Scrypto types that are encoded as custom SBOR types.
///
/// Any encode-able type in Scrypto library that requires special interpretation
/// must be declared as a custom type.
///
/// Custom types must be encoded as `[length + bytes]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScryptoType {
    // component
    PackageAddress,
    ComponentAddress,
    LazyMap,

    // crypto
    Hash,
    EcdsaPublicKey,
    EcdsaSignature,

    // math
    Decimal,

    // resource,
    Bucket,
    Proof,
    Vault,
    NonFungibleId,
    NonFungibleAddress,
    ResourceAddress,
}

// Need to update `scrypto-derive/src/import.rs` after changing the table below
const MAPPING: [(ScryptoType, u8, &str); 13] = [
    (ScryptoType::PackageAddress, 0x80, "PackageAddress"),
    (ScryptoType::ComponentAddress, 0x81, "ComponentAddress"),
    (ScryptoType::LazyMap, 0x82, "LazyMap"),
    (ScryptoType::Hash, 0x90, "Hash"),
    (ScryptoType::EcdsaPublicKey, 0x91, "EcdsaPublicKey"),
    (ScryptoType::EcdsaSignature, 0x92, "EcdsaSignature"),
    (ScryptoType::Decimal, 0xa1, "Decimal"),
    (ScryptoType::Bucket, 0xb1, "Bucket"),
    (ScryptoType::Proof, 0xb2, "Proof"),
    (ScryptoType::Vault, 0xb3, "Vault"),
    (ScryptoType::NonFungibleId, 0xb4, "NonFungibleId"),
    (ScryptoType::NonFungibleAddress, 0xb5, "NonFungibleAddress"),
    (ScryptoType::ResourceAddress, 0xb6, "ResourceAddress"),
];

impl ScryptoType {
    // TODO: optimize to get rid of loops

    pub fn from_id(id: u8) -> Option<ScryptoType> {
        MAPPING.iter().filter(|e| e.1 == id).map(|e| e.0).next()
    }

    pub fn from_name(name: &str) -> Option<ScryptoType> {
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

/// A validator the check a Scrypto-specific value.
pub struct ScryptoTypeValidator {
    pub buckets: Vec<Bucket>,
    pub proofs: Vec<Proof>,
    pub vaults: Vec<Vault>,
    pub lazy_maps: Vec<LazyMap<(), ()>>,
}

/// Represents an error when validating a Scrypto-specific value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoTypeValidationError {
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

impl ScryptoTypeValidator {
    pub fn new() -> Self {
        Self {
            buckets: Vec::new(),
            proofs: Vec::new(),
            vaults: Vec::new(),
            lazy_maps: Vec::new(),
        }
    }
}

impl CustomValueVisitor for ScryptoTypeValidator {
    type Err = ScryptoTypeValidationError;

    fn visit(&mut self, type_id: u8, data: &[u8]) -> Result<(), Self::Err> {
        match ScryptoType::from_id(type_id).ok_or(Self::Err::InvalidTypeId(type_id))? {
            ScryptoType::PackageAddress => {
                PackageAddress::try_from(data)
                    .map_err(ScryptoTypeValidationError::InvalidPackageAddress)?;
            }
            ScryptoType::ComponentAddress => {
                ComponentAddress::try_from(data)
                    .map_err(ScryptoTypeValidationError::InvalidComponentAddress)?;
            }
            ScryptoType::LazyMap => {
                self.lazy_maps.push(
                    LazyMap::try_from(data).map_err(ScryptoTypeValidationError::InvalidLazyMap)?,
                );
            }
            ScryptoType::Hash => {
                Hash::try_from(data).map_err(ScryptoTypeValidationError::InvalidHash)?;
            }
            ScryptoType::EcdsaPublicKey => {
                EcdsaPublicKey::try_from(data)
                    .map_err(ScryptoTypeValidationError::InvalidEcdsaPublicKey)?;
            }
            ScryptoType::EcdsaSignature => {
                EcdsaSignature::try_from(data)
                    .map_err(ScryptoTypeValidationError::InvalidEcdsaSignature)?;
            }
            ScryptoType::Decimal => {
                Decimal::try_from(data).map_err(ScryptoTypeValidationError::InvalidDecimal)?;
            }
            ScryptoType::Bucket => {
                self.buckets.push(
                    Bucket::try_from(data).map_err(ScryptoTypeValidationError::InvalidBucket)?,
                );
            }
            ScryptoType::Proof => {
                self.proofs
                    .push(Proof::try_from(data).map_err(ScryptoTypeValidationError::InvalidProof)?);
            }
            ScryptoType::Vault => {
                self.vaults
                    .push(Vault::try_from(data).map_err(ScryptoTypeValidationError::InvalidVault)?);
            }
            ScryptoType::NonFungibleId => {
                NonFungibleId::try_from(data)
                    .map_err(ScryptoTypeValidationError::InvalidNonFungibleId)?;
            }
            ScryptoType::NonFungibleAddress => {
                NonFungibleAddress::try_from(data)
                    .map_err(ScryptoTypeValidationError::InvalidNonFungibleAddress)?;
            }
            ScryptoType::ResourceAddress => {
                ResourceAddress::try_from(data)
                    .map_err(ScryptoTypeValidationError::InvalidResourceAddress)?;
            }
        }
        Ok(())
    }
}

/// A formatter that formats a Scrypto type.
pub struct ScryptoTypeFormatter {}

impl ScryptoTypeFormatter {
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
