use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RESOURCE_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::NonFungibleData;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use runtime::LocalAuthZone;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use scrypto::engine::scrypto_env::ScryptoEnv;

use crate::resource::*;
use crate::*;

pub trait ScryptoUncheckedProof {
    // Apply basic resource address check and converts self into `ValidatedProof`.
    fn check(self, resource_address: ResourceAddress) -> ValidatedProof;

    // Converts self into `ValidatedProof` with no check.
    fn no_check(self) -> ValidatedProof;

    fn resource_address(&self) -> ResourceAddress;

    fn drop(self);

    fn clone(&self) -> Self;

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;
}

pub trait ScryptoProof {
    /// Check if the proof satisfies the given `ProofValidation`
    fn validate(&self, validation_mode: ProofValidation) -> bool;

    fn amount(&self) -> Decimal;

    fn resource_address(&self) -> ResourceAddress;

    fn drop(self);

    fn clone(&self) -> Self;

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;
}

pub trait ScryptoFungibleProof {}

pub trait ScryptoNonFungibleProof {
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId>;

    fn non_fungible_local_id(&self) -> NonFungibleLocalId;

    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;

    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;
}

/// Represents an address-checked proof
///
/// This may become unnecessary when `Proof<X>` is supported.
///
// TODO: cache resource address in `ValidatedProof`!
#[derive(Debug, PartialEq, Eq, Hash, ScryptoSbor)]
#[sbor(transparent)]
pub struct ValidatedProof(pub Proof);

impl ValidatedProof {}

impl From<ValidatedProof> for Proof {
    fn from(value: ValidatedProof) -> Self {
        value.0
    }
}

impl ScryptoUncheckedProof for Proof {
    fn check(self, expected_resource_address: ResourceAddress) -> ValidatedProof {
        assert_eq!(self.resource_address(), expected_resource_address);
        ValidatedProof(self)
    }

    fn no_check(self) -> ValidatedProof {
        ValidatedProof(self)
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
        // TODO: Clean this up
        let info = env.get_object_info(self.0.as_node_id()).unwrap();
        env.call_function(
            RESOURCE_PACKAGE,
            info.blueprint.blueprint_name.as_str(),
            PROOF_DROP_IDENT,
            scrypto_encode(&ProofDropInput {
                proof: Proof(self.0),
            })
            .unwrap(),
        )
        .unwrap();
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

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        LocalAuthZone::push(self.clone());
        let output = f();
        LocalAuthZone::pop().drop();
        output
    }
}

impl ScryptoProof for ValidatedProof {
    fn amount(&self) -> Decimal {
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

    fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
    }

    fn drop(self) {
        self.0.drop()
    }

    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        self.0.authorize(f)
    }

    fn validate(&self, validation: ProofValidation) -> bool {
        match validation {
            ProofValidation::Contains(resource_address) => {
                self.resource_address().eq(&resource_address)
            }
            ProofValidation::ContainsNonFungible(non_fungible_global_id) => {
                self.resource_address()
                    .eq(&non_fungible_global_id.resource_address())
                    && self
                        .non_fungible_local_ids()
                        .contains(non_fungible_global_id.local_id())
            }
            ProofValidation::ContainsNonFungibles(resource_address, local_ids) => {
                self.resource_address().eq(&resource_address)
                    && self.non_fungible_local_ids().is_superset(&local_ids)
            }
            ProofValidation::ContainsAmount(resource_address, amount) => {
                self.resource_address().eq(&resource_address) && self.amount() >= amount
            }
            ProofValidation::ContainsAnyOf(resource_addresses) => {
                resource_addresses.contains(&self.resource_address())
            }
        }
    }
}

impl ScryptoNonFungibleProof for ValidatedProof {
    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible proof.
    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
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
    fn non_fungible_local_id(&self) -> NonFungibleLocalId {
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
    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT proof");
        }
        non_fungibles.into_iter().next().unwrap()
    }

    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId> {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT,
                scrypto_encode(&NonFungibleProofGetLocalIdsInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}
