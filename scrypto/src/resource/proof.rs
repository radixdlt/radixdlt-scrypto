use crate::resource::*;
use crate::runtime::Runtime;
use crate::*;
use radix_common::prelude::*;
use radix_common::traits::NonFungibleData;
use radix_engine_interface::blueprints::resource::*;
use runtime::LocalAuthZone;
use scrypto::engine::scrypto_env::ScryptoVmV1Api;

// Different from the native SDK, in Scrypto we use `CheckedProof`, `CheckedFungibleProof`
// and `CheckedNonFungibleProof` (instead of `Proof`/`FungibleProof`/`NonFungibleProof`)
// to prevent developers from reading proof states (and having business logic relying on them)
// without checking the resource address.

//========
// Traits
//========

pub trait ScryptoUncheckedProof {
    type CheckedProofType;
    type ResourceManagerType;

    /// Checks the resource address of this proof and panics if it's unexpected.
    fn check(self, expected_resource_address: ResourceAddress) -> Self::CheckedProofType;

    /// Checks the resource address of this proof and panics with custom error message if it's unexpected.
    fn check_with_message<S: ToString>(
        self,
        expected_resource_address: ResourceAddress,
        custom_error_message: S,
    ) -> Self::CheckedProofType;

    /// Skips checking and converts this proof into a "checked" proof.
    ///
    /// # Warning!
    /// Be sure to validate the resource address before reading data from the proof
    /// in your custom validation logic!
    fn skip_checking(self) -> Self::CheckedProofType;

    fn resource_address(&self) -> ResourceAddress;

    fn resource_manager(&self) -> Self::ResourceManagerType;

    fn drop(self);

    fn clone(&self) -> Self;

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;
}

pub trait ScryptoProof {
    type ResourceManagerType;

    fn contains_amount(&self, amount: Decimal) -> bool;

    fn amount(&self) -> Decimal;

    fn resource_address(&self) -> ResourceAddress;

    fn resource_manager(&self) -> Self::ResourceManagerType;

    fn drop(self);

    fn clone(&self) -> Self;

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;
}

pub trait ScryptoGenericProof {
    fn as_fungible(&self) -> CheckedFungibleProof;

    fn as_non_fungible(&self) -> CheckedNonFungibleProof;
}

pub trait ScryptoFungibleProof {}

pub trait ScryptoNonFungibleProof {
    fn contains_non_fungible(&self, id: &NonFungibleLocalId) -> bool;

    fn contains_non_fungibles(&self, ids: &IndexSet<NonFungibleLocalId>) -> bool;

    fn non_fungible_local_ids(&self) -> IndexSet<NonFungibleLocalId>;

    fn non_fungible_local_id(&self) -> NonFungibleLocalId;

    fn non_fungible_global_id(&self) -> NonFungibleGlobalId;

    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;

    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;
}

//=====================
// Checked proof types
//=====================

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

//=================
// Unchecked proof
//=================

impl ScryptoUncheckedProof for Proof {
    type CheckedProofType = CheckedProof;
    type ResourceManagerType = ResourceManager;

    fn check(self, expected_resource_address: ResourceAddress) -> CheckedProof {
        let actual_resource_address = self.resource_address();

        if actual_resource_address != expected_resource_address {
            Runtime::panic(format!(
                "Invalid proof: Expected {:?}, but got {:?}",
                expected_resource_address, actual_resource_address
            ))
        }

        CheckedProof(self)
    }

    fn check_with_message<S: ToString>(
        self,
        expected_resource_address: ResourceAddress,
        custom_error_message: S,
    ) -> CheckedProof {
        let actual_resource_address = self.resource_address();

        if actual_resource_address != expected_resource_address {
            Runtime::panic(custom_error_message.to_string())
        }

        CheckedProof(self)
    }

    fn skip_checking(self) -> CheckedProof {
        CheckedProof(self)
    }

    fn resource_address(&self) -> ResourceAddress {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            PROOF_GET_RESOURCE_ADDRESS_IDENT,
            scrypto_encode(&ProofGetResourceAddressInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
    }

    fn drop(self) {
        ScryptoVmV1Api::blueprint_call(
            RESOURCE_PACKAGE,
            if ScryptoVmV1Api::object_instance_of(
                self.0.as_node_id(),
                &BlueprintId {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: FUNGIBLE_PROOF_BLUEPRINT.to_owned(),
                },
            ) {
                FUNGIBLE_PROOF_BLUEPRINT
            } else {
                NON_FUNGIBLE_PROOF_BLUEPRINT
            },
            PROOF_DROP_IDENT,
            scrypto_encode(&ProofDropInput {
                proof: Proof(self.0),
            })
            .unwrap(),
        );
    }

    fn clone(&self) -> Self {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            PROOF_CLONE_IDENT,
            scrypto_encode(&ProofCloneInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        LocalAuthZone::push(self.clone());
        let output = f();
        LocalAuthZone::pop()
            .expect("Authorized closure changed auth zone proof stack")
            .drop();
        output
    }
}

impl ScryptoUncheckedProof for FungibleProof {
    type CheckedProofType = CheckedFungibleProof;
    type ResourceManagerType = FungibleResourceManager;

    fn check(self, expected_resource_address: ResourceAddress) -> Self::CheckedProofType {
        CheckedFungibleProof(Proof::check(self.0, expected_resource_address))
    }

    fn check_with_message<S: ToString>(
        self,
        expected_resource_address: ResourceAddress,
        custom_error_message: S,
    ) -> Self::CheckedProofType {
        CheckedFungibleProof(Proof::check_with_message(
            self.0,
            expected_resource_address,
            custom_error_message,
        ))
    }

    fn skip_checking(self) -> Self::CheckedProofType {
        CheckedFungibleProof(Proof::skip_checking(self.0))
    }

    fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
    }

    fn drop(self) {
        self.0.drop()
    }

    fn clone(&self) -> Self {
        FungibleProof(self.0.clone())
    }

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        self.0.authorize(f)
    }
}

impl ScryptoUncheckedProof for NonFungibleProof {
    type CheckedProofType = CheckedNonFungibleProof;
    type ResourceManagerType = NonFungibleResourceManager;

    fn check(self, expected_resource_address: ResourceAddress) -> Self::CheckedProofType {
        CheckedNonFungibleProof(Proof::check(self.0, expected_resource_address))
    }

    fn check_with_message<S: ToString>(
        self,
        expected_resource_address: ResourceAddress,
        custom_error_message: S,
    ) -> Self::CheckedProofType {
        CheckedNonFungibleProof(Proof::check_with_message(
            self.0,
            expected_resource_address,
            custom_error_message,
        ))
    }

    fn skip_checking(self) -> Self::CheckedProofType {
        CheckedNonFungibleProof(Proof::skip_checking(self.0))
    }

    fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
    }

    fn drop(self) {
        self.0.drop()
    }

    fn clone(&self) -> Self {
        NonFungibleProof(self.0.clone())
    }

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        self.0.authorize(f)
    }
}

//===================
// Any checked Proof
//===================

impl ScryptoProof for CheckedProof {
    type ResourceManagerType = ResourceManager;

    fn contains_amount(&self, amount: Decimal) -> bool {
        self.amount() >= amount
    }

    fn amount(&self) -> Decimal {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            PROOF_GET_AMOUNT_IDENT,
            scrypto_encode(&ProofGetAmountInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
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
}

impl ScryptoGenericProof for CheckedProof {
    fn as_fungible(&self) -> CheckedFungibleProof {
        assert!(
            self.resource_address()
                .as_node_id()
                .is_global_fungible_resource_manager(),
            "Not a fungible proof"
        );
        CheckedFungibleProof(Self(Proof(self.0 .0)))
    }

    fn as_non_fungible(&self) -> CheckedNonFungibleProof {
        assert!(
            self.resource_address()
                .as_node_id()
                .is_global_non_fungible_resource_manager(),
            "Not a non-fungible proof"
        );
        CheckedNonFungibleProof(Self(Proof(self.0 .0)))
    }
}

//========================
// Checked fungible proof
//========================

impl ScryptoProof for CheckedFungibleProof {
    type ResourceManagerType = FungibleResourceManager;

    fn contains_amount(&self, amount: Decimal) -> bool {
        self.0.contains_amount(amount)
    }

    fn amount(&self) -> Decimal {
        self.0.amount()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
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
}

impl ScryptoFungibleProof for CheckedFungibleProof {}

//============================
// Checked non-fungible proof
//============================

impl ScryptoProof for CheckedNonFungibleProof {
    type ResourceManagerType = NonFungibleResourceManager;

    fn contains_amount(&self, amount: Decimal) -> bool {
        self.0.contains_amount(amount)
    }

    fn amount(&self) -> Decimal {
        self.0.amount()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
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
}

impl ScryptoNonFungibleProof for CheckedNonFungibleProof {
    fn contains_non_fungible(&self, id: &NonFungibleLocalId) -> bool {
        self.non_fungible_local_ids().contains(id)
    }

    fn contains_non_fungibles(&self, ids: &IndexSet<NonFungibleLocalId>) -> bool {
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

    /// Returns the non-fungible local id if this is a singleton non-fungible proof.
    ///
    /// # Panics
    /// Panics if this is not a singleton proof
    fn non_fungible_local_id(&self) -> NonFungibleLocalId {
        let non_fungible_local_ids = self.non_fungible_local_ids();
        if non_fungible_local_ids.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        self.non_fungible_local_ids().into_iter().next().unwrap()
    }

    /// Returns the non-fungible global id if this is a singleton non-fungible proof.
    ///
    /// # Panics
    /// Panics if this is not a singleton proof
    fn non_fungible_global_id(&self) -> NonFungibleGlobalId {
        NonFungibleGlobalId::new(self.resource_address(), self.non_fungible_local_id())
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

    fn non_fungible_local_ids(&self) -> IndexSet<NonFungibleLocalId> {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0 .0.as_node_id(),
            NON_FUNGIBLE_PROOF_GET_LOCAL_IDS_IDENT,
            scrypto_encode(&NonFungibleProofGetLocalIdsInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }
}
