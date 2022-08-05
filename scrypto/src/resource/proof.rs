use sbor::rust::collections::BTreeSet;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::core::Receiver;
use crate::engine::types::RENodeId;
use crate::engine::{api::*, call_engine, types::ProofId};
use crate::math::*;
use crate::misc::*;
use crate::resource::*;
use crate::sfunctions;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ConsumingProofDropInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ProofGetAmountInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ProofGetNonFungibleIdsInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ProofGetResourceAddressInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ProofCloneInput {}

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

impl Clone for Proof {
    sfunctions! {
        Receiver::ProofRef(self.0) => {
            fn clone(&self) -> Self {
                ProofCloneInput {}
            }
        }
    }
}

impl Proof {
    /// Validates a `Proof`'s resource address creating a `ValidatedProof` if the validation succeeds.
    ///
    /// This method takes ownership of the proof and validates that its resource address matches that expected by the
    /// caller. If the validation is successful, then a `ValidatedProof` is returned, otherwise, a `ValidateProofError`
    /// is returned.
    ///
    /// # Example:
    ///
    /// ```ignore
    /// let proof: Proof = bucket.create_proof();
    /// match proof.validate_proof(admin_badge_resource_address) {
    ///     Ok(validated_proof) => {
    ///         info!(
    ///             "Validation successful. Proof has a resource address of {} and amount of {}",
    ///             validated_proof.resource_address(),
    ///             validated_proof.amount(),
    ///         );
    ///     },
    ///     Err(error) => {
    ///         info!("Error validating proof: {:?}", error);
    ///     },
    /// }
    /// ```
    pub fn validate_proof<T>(
        self,
        validation_mode: T,
    ) -> Result<ValidatedProof, (Self, ProofValidationError)>
    where
        T: Into<ProofValidationMode>,
    {
        let validation_mode: ProofValidationMode = validation_mode.into();
        match self.validate(validation_mode) {
            Ok(()) => Ok(ValidatedProof(self)),
            Err(error) => Err((self, error)),
        }
    }

    /// Skips the validation process of the proof producing a validated proof **WITHOUT** performing any validation.
    ///
    /// # WARNING:
    ///
    /// This method skips the validation of the resource address of the proof. Therefore, the data, or `NonFungibleId`
    /// of of the returned `ValidatedProof` should **NOT** be trusted as the proof could potentially belong to any
    /// resource address. If you call this method, you should perform your own validation.
    pub fn unsafe_skip_proof_validation(self) -> ValidatedProof {
        ValidatedProof(self)
    }

    /// Converts a `ValidatedProof` into a `Proof`.
    pub fn from_validated_proof(validated_proof: ValidatedProof) -> Self {
        validated_proof.into()
    }

    fn validate(&self, validation_mode: ProofValidationMode) -> Result<(), ProofValidationError> {
        match validation_mode {
            ProofValidationMode::ValidateResourceAddress(resource_address) => {
                self.validate_resource_address(resource_address)?;
                Ok(())
            }
            ProofValidationMode::ValidateResourceAddressBelongsTo(resource_addresses) => {
                self.validate_resource_address_belongs_to(&resource_addresses)?;
                Ok(())
            }
            ProofValidationMode::ValidateContainsNonFungible(non_fungible_address) => {
                self.validate_resource_address(non_fungible_address.resource_address())?;
                self.validate_contains_non_fungible_id(non_fungible_address.non_fungible_id())?;
                Ok(())
            }
            ProofValidationMode::ValidateContainsNonFungibles(
                resource_address,
                non_fungible_ids,
            ) => {
                self.validate_resource_address(resource_address)?;
                self.validate_contains_non_fungible_ids(&non_fungible_ids)?;
                Ok(())
            }
            ProofValidationMode::ValidateContainsAmount(resource_address, amount) => {
                self.validate_resource_address(resource_address)?;
                self.validate_contains_amount(amount)?;
                Ok(())
            }
        }
    }

    fn validate_resource_address(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<(), ProofValidationError> {
        if self.resource_address() == resource_address {
            Ok(())
        } else {
            Err(ProofValidationError::InvalidResourceAddress(
                resource_address,
            ))
        }
    }

    fn validate_resource_address_belongs_to(
        &self,
        resource_addresses: &BTreeSet<ResourceAddress>,
    ) -> Result<(), ProofValidationError> {
        if resource_addresses.contains(&self.resource_address()) {
            Ok(())
        } else {
            Err(ProofValidationError::ResourceAddressDoesNotBelongToList)
        }
    }

    fn validate_contains_non_fungible_id(
        &self,
        non_fungible_id: NonFungibleId,
    ) -> Result<(), ProofValidationError> {
        if self.non_fungible_ids().get(&non_fungible_id).is_some() {
            Ok(())
        } else {
            Err(ProofValidationError::InvalidNonFungibleId(non_fungible_id))
        }
    }

    fn validate_contains_non_fungible_ids(
        &self,
        expected_non_fungible_ids: &BTreeSet<NonFungibleId>,
    ) -> Result<(), ProofValidationError> {
        let actual_non_fungible_ids = self.non_fungible_ids();
        let contains_all_non_fungible_ids = expected_non_fungible_ids
            .iter()
            .all(|non_fungible_id| actual_non_fungible_ids.get(non_fungible_id).is_some());
        if contains_all_non_fungible_ids {
            Ok(())
        } else {
            Err(ProofValidationError::NonFungibleIdsNotSubsetOfList)
        }
    }

    fn validate_contains_amount(&self, amount: Decimal) -> Result<(), ProofValidationError> {
        if self.amount() >= amount {
            Ok(())
        } else {
            Err(ProofValidationError::InvalidAmount(amount))
        }
    }

    sfunctions! {
        Receiver::ProofRef(self.0) => {
            fn amount(&self) -> Decimal {
                ProofGetAmountInput {}
            }
            fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
                ProofGetNonFungibleIdsInput {}
            }
            fn resource_address(&self) -> ResourceAddress {
                ProofGetResourceAddressInput {}
            }
        }
    }

    sfunctions! {
        Receiver::Consumed(RENodeId::Proof(self.0)) => {
            pub fn drop(self) -> () {
                ConsumingProofDropInput {}
            }
        }
    }
}

/// Represents a proof of owning some resource that has had its resource address validated.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ValidatedProof(pub(crate) Proof);

impl Clone for ValidatedProof {
    fn clone(&self) -> Self {
        ValidatedProof(self.0.clone())
    }
}

impl ValidatedProof {
    sfunctions! {
        Receiver::ProofRef(self.proof_id()) => {
            pub fn amount(&self) -> Decimal {
                ProofGetAmountInput {}
            }
            pub fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
                ProofGetNonFungibleIdsInput {}
            }
            pub fn resource_address(&self) -> ResourceAddress {
                ProofGetResourceAddressInput {}
            }
        }
    }

    pub fn drop(self) {
        self.0.drop()
    }

    /// Whether this proof includes an ownership proof of any of the given resource.
    pub fn contains(&self, resource_address: ResourceAddress) -> bool {
        self.resource_address() == resource_address
    }

    /// Whether this proof includes an ownership proof of at least the given amount of resource.
    pub fn contains_resource(&self, amount: Decimal, resource_address: ResourceAddress) -> bool {
        self.resource_address() == resource_address && self.amount() > amount
    }

    /// Whether this proof includes an ownership proof of the given non-fungible.
    pub fn contains_non_fungible(&self, non_fungible_address: &NonFungibleAddress) -> bool {
        if self.resource_address() != non_fungible_address.resource_address() {
            return false;
        }

        self.non_fungible_ids()
            .iter()
            .any(|k| k.eq(&non_fungible_address.non_fungible_id()))
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible proof.
    pub fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self.non_fungible_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleAddress::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns a singleton non-fungible.
    ///
    /// # Panics
    /// Panics if this is not a singleton proof
    pub fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT proof");
        }
        non_fungibles.into_iter().next().unwrap()
    }

    /// Checks if the referenced bucket is empty.
    pub fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    fn proof_id(&self) -> ProofId {
        self.0 .0
    }
}

impl Into<Proof> for ValidatedProof {
    fn into(self) -> Proof {
        self.0
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
    InvalidNonFungibleId(NonFungibleId),
    NonFungibleIdsNotSubsetOfList,
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

impl ValidatedProof {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

// Note: Only `Proof` is a Scrypto type, `ValidatedProof` is not. This is because `ValidatedProof`s doesn't need to
// implement the sbor::Encode and sbor::Decode traits as they are not meant to be used as arguments and returns to and
// from methods. They are meant ot be used inside methods.

scrypto_type!(Proof, ScryptoType::Proof, Vec::new());
