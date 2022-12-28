use sbor::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

use crate::abi::*;
use crate::api::{api::*, types::*};
use crate::data::ScryptoCustomTypeId;
use crate::math::*;
use crate::scrypto_type;
use crate::wasm::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct ProofGetAmountInvocation {
    pub receiver: ProofId,
}

impl Invocation for ProofGetAmountInvocation {
    type Output = Decimal;
}

impl SerializableInvocation for ProofGetAmountInvocation {
    type ScryptoOutput = Decimal;
}

impl Into<SerializedInvocation> for ProofGetAmountInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Proof(ProofInvocation::GetAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct ProofGetNonFungibleIdsInvocation {
    pub receiver: ProofId,
}

impl Invocation for ProofGetNonFungibleIdsInvocation {
    type Output = BTreeSet<NonFungibleId>;
}

impl SerializableInvocation for ProofGetNonFungibleIdsInvocation {
    type ScryptoOutput = BTreeSet<NonFungibleId>;
}

impl Into<SerializedInvocation> for ProofGetNonFungibleIdsInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Proof(ProofInvocation::GetNonFungibleIds(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct ProofGetResourceAddressInvocation {
    pub receiver: ProofId,
}

impl Invocation for ProofGetResourceAddressInvocation {
    type Output = ResourceAddress;
}

impl SerializableInvocation for ProofGetResourceAddressInvocation {
    type ScryptoOutput = ResourceAddress;
}

impl Into<SerializedInvocation> for ProofGetResourceAddressInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Proof(ProofInvocation::GetResourceAddress(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct ProofCloneInvocation {
    pub receiver: ProofId,
}

impl Invocation for ProofCloneInvocation {
    type Output = Proof;
}

impl SerializableInvocation for ProofCloneInvocation {
    type ScryptoOutput = Proof;
}

impl Into<SerializedInvocation> for ProofCloneInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::Proof(ProofInvocation::Clone(self)).into()
    }
}

/// Represents a proof of owning some resource.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Proof(pub ProofId);

// TODO: Evaluate if we should have a ProofValidationModeBuilder to construct more complex validation modes.
/// Specifies the validation mode that should be used for validating a `Proof`.
pub enum ProofValidationMode {
    /// Specifies that the `Proof` should be validated against a single `ResourceAddress`.
    ValidateResourceAddress(ResourceAddress),

    /// Specifies that the `Proof` should have its resource address validated against a set of `ResourceAddress`es. If
    /// the `Proof`'s resource address belongs to the set, then its valid.
    ValidateResourceAddressBelongsTo(BTreeSet<ResourceAddress>),

    /// Specifies that the `Proof` should be validating for containing a specific `NonFungibleAddress`.
    ValidateContainsNonFungible(NonFungibleAddress),

    /// Specifies that the `Proof` should be validated against a single resource address and a set of `NonFungibleId`s
    /// to ensure that the `Proof` contains all of the NonFungibles in the set.
    ValidateContainsNonFungibles(ResourceAddress, BTreeSet<NonFungibleId>),

    /// Specifies that the `Proof` should be validated for the amount of resources that it contains.
    ValidateContainsAmount(ResourceAddress, Decimal),
}

impl From<ResourceAddress> for ProofValidationMode {
    fn from(resource_address: ResourceAddress) -> Self {
        Self::ValidateResourceAddress(resource_address)
    }
}

impl From<NonFungibleAddress> for ProofValidationMode {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        Self::ValidateContainsNonFungible(non_fungible_address)
    }
}

//========
// error
//========

/// Represents an error when decoding proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseProofError {
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseProofError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseProofError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents an error when validating proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofValidationError {
    InvalidResourceAddress(ResourceAddress),
    ResourceAddressDoesNotBelongToList,
    DoesNotContainOneNonFungible,
    NonFungibleIdNotFound,
    InvalidAmount(Decimal),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ProofValidationError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ProofValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Proof {
    type Error = ParseProofError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            4 => Ok(Self(u32::from_le_bytes(copy_u8_array(slice)))),
            _ => Err(ParseProofError::InvalidLength(slice.len())),
        }
    }
}

impl Proof {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

// Note: Only `Proof` is a Scrypto type, `ValidatedProof` is not. This is because `ValidatedProof`s doesn't need to
// implement the sbor::Encode and sbor::Decode traits as they are not meant to be used as arguments and returns to and
// from methods. They are meant ot be used inside methods.
scrypto_type!(Proof, ScryptoCustomTypeId::Proof, Type::Proof, 4);
