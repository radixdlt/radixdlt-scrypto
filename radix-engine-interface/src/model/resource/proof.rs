use sbor::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::*;

use crate::abi::*;
use crate::api::{api::*, types::*};
use crate::data::types::Own;
use crate::data::ScryptoCustomValueKind;
use crate::math::*;
use crate::wasm::*;

#[derive(Debug, Clone, Eq, PartialEq, Categorize, Encode, Decode)]
pub struct ProofGetAmountInvocation {
    pub receiver: ProofId,
}

impl Invocation for ProofGetAmountInvocation {
    type Output = Decimal;
}

impl SerializableInvocation for ProofGetAmountInvocation {
    type ScryptoOutput = Decimal;
}

impl Into<CallTableInvocation> for ProofGetAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Proof(ProofInvocation::GetAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Categorize, Encode, Decode)]
pub struct ProofGetNonFungibleIdsInvocation {
    pub receiver: ProofId,
}

impl Invocation for ProofGetNonFungibleIdsInvocation {
    type Output = BTreeSet<NonFungibleId>;
}

impl SerializableInvocation for ProofGetNonFungibleIdsInvocation {
    type ScryptoOutput = BTreeSet<NonFungibleId>;
}

impl Into<CallTableInvocation> for ProofGetNonFungibleIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Proof(ProofInvocation::GetNonFungibleIds(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Categorize, Encode, Decode)]
pub struct ProofGetResourceAddressInvocation {
    pub receiver: ProofId,
}

impl Invocation for ProofGetResourceAddressInvocation {
    type Output = ResourceAddress;
}

impl SerializableInvocation for ProofGetResourceAddressInvocation {
    type ScryptoOutput = ResourceAddress;
}

impl Into<CallTableInvocation> for ProofGetResourceAddressInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Proof(ProofInvocation::GetResourceAddress(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Categorize, Encode, Decode)]
pub struct ProofCloneInvocation {
    pub receiver: ProofId,
}

impl Invocation for ProofCloneInvocation {
    type Output = Proof;
}

impl SerializableInvocation for ProofCloneInvocation {
    type ScryptoOutput = Proof;
}

impl Into<CallTableInvocation> for ProofCloneInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::Proof(ProofInvocation::Clone(self)).into()
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Proof(pub ProofId); // scrypto stub

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Proof {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Proof {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Own::Proof(self.0).encode_body(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Proof {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let o = Own::decode_body_with_value_kind(decoder, value_kind)?;
        match o {
            Own::Proof(proof_id) => Ok(Self(proof_id)),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for Proof {
    fn describe() -> scrypto_abi::Type {
        Type::Proof
    }
}
