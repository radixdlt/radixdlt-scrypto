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

// Different from the native SDK, in Scrypto we use `CheckedProof`, `CheckedFungibleProof`
// and `CheckedNonFungibleProof` (instead of `Proof`/`FungibleProof`/`NonFungibleProof`)
// to prevent developers from reading proof states (and having business logic relying on them)
// without checking the resource address.

pub trait ScryptoUncheckedProof {
    // Apply basic resource address check and converts self into `CheckedProof`.
    fn check(self, resource_address: ResourceAddress) -> CheckedProof;

    // Converts self into `CheckedProof` with no check.
    fn no_check(self) -> CheckedProof;

    fn resource_address(&self) -> ResourceAddress;

    fn drop(self);

    fn clone(&self) -> Self;

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;
}

pub trait ScryptoProof {
    fn contains_amount(&self, amount: Decimal) -> bool;

    fn amount(&self) -> Decimal;

    fn resource_address(&self) -> ResourceAddress;

    fn drop(self);

    fn clone(&self) -> Self;

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;

    fn as_fungible_proof(&self) -> CheckedFungibleProof;

    fn as_no_fungible_proof(&self) -> CheckedNonFungibleProof;
}

pub trait ScryptoFungibleProof {}

pub trait ScryptoNonFungibleProof {
    fn contains_non_fungible(&self, id: &NonFungibleLocalId) -> bool;

    fn contains_non_fungibles(&self, ids: &BTreeSet<NonFungibleLocalId>) -> bool;

    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId>;

    fn non_fungible_local_id(&self) -> NonFungibleLocalId;

    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;

    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;
}

/// Represents an address-checked proof
///
/// This may become unnecessary when `Proof<X>` is supported.
///
// TODO: cache resource address in `CheckedProof`!
#[derive(Debug, PartialEq, Eq, Hash, ScryptoSbor)]
#[sbor(transparent)]
pub struct CheckedProof(pub Proof);

#[derive(Debug, PartialEq, Eq, Hash, ScryptoSbor)]
#[sbor(transparent)]
pub struct CheckedFungibleProof(pub CheckedProof);

#[derive(Debug, PartialEq, Eq, Hash, ScryptoSbor)]
#[sbor(transparent)]
pub struct CheckedNonFungibleProof(pub CheckedProof);

impl From<CheckedFungibleProof> for CheckedProof {
    fn from(value: CheckedFungibleProof) -> Self {
        value.0
    }
}

impl From<CheckedNonFungibleProof> for CheckedProof {
    fn from(value: CheckedNonFungibleProof) -> Self {
        value.0
    }
}

impl AsRef<CheckedProof> for CheckedProof {
    fn as_ref(&self) -> &CheckedProof {
        self
    }
}

impl AsRef<CheckedProof> for CheckedFungibleProof {
    fn as_ref(&self) -> &CheckedProof {
        &self.0
    }
}

impl AsRef<CheckedProof> for CheckedNonFungibleProof {
    fn as_ref(&self) -> &CheckedProof {
        &self.0
    }
}

impl From<CheckedProof> for Proof {
    fn from(value: CheckedProof) -> Self {
        value.0
    }
}

impl ScryptoUncheckedProof for Proof {
    fn check(self, expected_resource_address: ResourceAddress) -> CheckedProof {
        assert_eq!(self.resource_address(), expected_resource_address);
        CheckedProof(self)
    }

    fn no_check(self) -> CheckedProof {
        CheckedProof(self)
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

impl ScryptoProof for CheckedProof {
    fn contains_amount(&self, amount: Decimal) -> bool {
        self.amount() >= amount
    }

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

    // TODO: should we check fungibility here?
    // Currently, it will fail at runtime when invoking fungible/non-fungible methods

    fn as_fungible_proof(&self) -> CheckedFungibleProof {
        CheckedFungibleProof(CheckedProof(Proof(self.as_ref().0 .0)))
    }

    fn as_no_fungible_proof(&self) -> CheckedNonFungibleProof {
        CheckedNonFungibleProof(CheckedProof(Proof(self.as_ref().0 .0)))
    }
}

impl ScryptoFungibleProof for CheckedFungibleProof {}

impl ScryptoNonFungibleProof for CheckedNonFungibleProof {
    fn contains_non_fungible(&self, id: &NonFungibleLocalId) -> bool {
        self.non_fungible_local_ids().contains(&id)
    }

    fn contains_non_fungibles(&self, ids: &BTreeSet<NonFungibleLocalId>) -> bool {
        self.non_fungible_local_ids().is_superset(&ids)
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible proof.
    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.0.resource_address();
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
                self.0 .0 .0.as_node_id(),
                NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT,
                scrypto_encode(&NonFungibleProofGetLocalIdsInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}
