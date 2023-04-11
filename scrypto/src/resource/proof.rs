use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RESOURCE_MANAGER_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::NonFungibleData;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use scrypto::engine::scrypto_env::ScryptoEnv;

use crate::resource::*;
use crate::*;

pub trait ScryptoProof: Sized {
    fn clone(&self) -> Self;
    fn validate_proof<T>(
        self,
        validation_mode: T,
    ) -> Result<ValidatedProof, (Self, ProofValidationError)>
    where
        T: Into<ProofValidationMode>;
    fn unsafe_skip_proof_validation(self) -> ValidatedProof;
    fn from_validated_proof(validated_proof: ValidatedProof) -> Self;
    fn validate(&self, validation_mode: ProofValidationMode) -> Result<(), ProofValidationError>;
    fn validate_resource_address(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<(), ProofValidationError>;
    fn validate_resource_address_belongs_to(
        &self,
        resource_addresses: &BTreeSet<ResourceAddress>,
    ) -> Result<(), ProofValidationError>;
    fn validate_contains_non_fungible_local_id(
        &self,
        non_fungible_local_id: &NonFungibleLocalId,
    ) -> Result<(), ProofValidationError>;
    fn validate_contains_non_fungible_local_ids(
        &self,
        expected_non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<(), ProofValidationError>;
    fn validate_contains_amount(&self, amount: Decimal) -> Result<(), ProofValidationError>;
    fn amount(&self) -> Decimal;
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId>;
    fn resource_address(&self) -> ResourceAddress;
    fn drop(self);
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;
}

impl ScryptoProof for Proof {
    /// Uses resources in this proof as authorization for an operation.
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        ComponentAuthZone::push(ScryptoProof::clone(self));
        let output = f();
        ComponentAuthZone::pop().drop();
        output
    }

    fn clone(&self) -> Self {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                PROOF_CLONE_IDENT,
                scrypto_encode(&ProofCloneInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

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
    fn validate_proof<T>(
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
    /// This method skips the validation of the resource address of the proof. Therefore, the data, or `NonFungibleLocalId`
    /// of of the returned `ValidatedProof` should **NOT** be trusted as the proof could potentially belong to any
    /// resource address. If you call this method, you should perform your own validation.
    fn unsafe_skip_proof_validation(self) -> ValidatedProof {
        ValidatedProof(self)
    }

    /// Converts a `ValidatedProof` into a `Proof`.
    fn from_validated_proof(validated_proof: ValidatedProof) -> Self {
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
            ProofValidationMode::ValidateContainsNonFungible(non_fungible_global_id) => {
                self.validate_resource_address(non_fungible_global_id.resource_address())?;
                self.validate_contains_non_fungible_local_id(non_fungible_global_id.local_id())?;
                Ok(())
            }
            ProofValidationMode::ValidateContainsNonFungibles(
                resource_address,
                non_fungible_local_ids,
            ) => {
                self.validate_resource_address(resource_address)?;
                self.validate_contains_non_fungible_local_ids(&non_fungible_local_ids)?;
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

    fn validate_contains_non_fungible_local_id(
        &self,
        non_fungible_local_id: &NonFungibleLocalId,
    ) -> Result<(), ProofValidationError> {
        if self
            .non_fungible_local_ids()
            .get(non_fungible_local_id)
            .is_some()
        {
            Ok(())
        } else {
            Err(ProofValidationError::NonFungibleLocalIdNotFound)
        }
    }

    fn validate_contains_non_fungible_local_ids(
        &self,
        expected_non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<(), ProofValidationError> {
        let actual_non_fungible_local_ids = self.non_fungible_local_ids();
        let contains_all_non_fungible_local_ids =
            expected_non_fungible_local_ids
                .iter()
                .all(|non_fungible_local_id| {
                    actual_non_fungible_local_ids
                        .get(non_fungible_local_id)
                        .is_some()
                });
        if contains_all_non_fungible_local_ids {
            Ok(())
        } else {
            Err(ProofValidationError::NonFungibleLocalIdNotFound)
        }
    }

    fn validate_contains_amount(&self, amount: Decimal) -> Result<(), ProofValidationError> {
        if self.amount() >= amount {
            Ok(())
        } else {
            Err(ProofValidationError::InvalidAmount(amount))
        }
    }

    fn amount(&self) -> Decimal {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                PROOF_GET_AMOUNT_IDENT,
                scrypto_encode(&ProofGetAmountInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId> {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                PROOF_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                scrypto_encode(&ProofGetNonFungibleLocalIdsInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn resource_address(&self) -> ResourceAddress {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                PROOF_GET_RESOURCE_ADDRESS_IDENT,
                scrypto_encode(&ProofGetResourceAddressInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn drop(self) {
        let mut env = ScryptoEnv;
        env.call_function(
            RESOURCE_MANAGER_PACKAGE,
            PROOF_BLUEPRINT,
            PROOF_DROP_IDENT,
            scrypto_encode(&ProofDropInput {
                proof: Proof(self.0),
            })
            .unwrap(),
        )
        .unwrap();
    }
}

/// Represents a proof of owning some resource that has had its resource address validated.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ValidatedProof(pub(crate) Proof);

#[cfg(target_arch = "wasm32")]
impl Clone for ValidatedProof {
    fn clone(&self) -> Self {
        ValidatedProof(ScryptoProof::clone(&self.0))
    }
}

impl ValidatedProof {
    pub fn amount(&self) -> Decimal {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                PROOF_GET_AMOUNT_IDENT,
                scrypto_encode(&ProofGetAmountInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId> {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                PROOF_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                scrypto_encode(&ProofGetNonFungibleLocalIdsInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn resource_address(&self) -> ResourceAddress {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                PROOF_GET_RESOURCE_ADDRESS_IDENT,
                scrypto_encode(&ProofGetResourceAddressInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    #[cfg(target_arch = "wasm32")]
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
    pub fn contains_non_fungible(&self, non_fungible_global_id: &NonFungibleGlobalId) -> bool {
        if self.resource_address() != non_fungible_global_id.resource_address() {
            return false;
        }

        self.non_fungible_local_ids()
            .iter()
            .any(|k| k.eq(&non_fungible_global_id.local_id()))
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible proof.
    pub fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self.non_fungible_local_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleGlobalId::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns a singleton non-fungible id
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    pub fn non_fungible_local_id(&self) -> NonFungibleLocalId {
        let non_fungible_local_ids = self.non_fungible_local_ids();
        if non_fungible_local_ids.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        self.non_fungible_local_ids().into_iter().next().unwrap()
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
}

impl Into<Proof> for ValidatedProof {
    fn into(self) -> Proof {
        self.0
    }
}
