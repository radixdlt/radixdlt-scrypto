use super::ScryptoUncheckedProof;
use crate::prelude::{
    FungibleResourceManager, NonFungibleResourceManager, ResourceManager,
    ScryptoResourceManagerStub,
};
use crate::resource::NonFungible;
use crate::runtime::LocalAuthZone;
use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_common::traits::NonFungibleData;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use sbor::rust::prelude::*;
use scrypto::engine::scrypto_env::ScryptoVmV1Api;

//=============
// Traits
//=============

pub trait ScryptoBucket {
    type ProofType;
    type ResourceManagerType;

    fn new(resource_address: ResourceAddress) -> Self;

    fn drop_empty(self);

    fn burn(self);

    fn create_proof_of_all(&self) -> Self::ProofType;

    fn resource_address(&self) -> ResourceAddress;

    fn resource_manager(&self) -> Self::ResourceManagerType;

    fn put(&mut self, other: Self) -> ();

    fn amount(&self) -> Decimal;

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self;

    fn take_advanced<A: Into<Decimal>>(
        &mut self,
        amount: A,
        withdraw_strategy: WithdrawStrategy,
    ) -> Self;

    fn is_empty(&self) -> bool;

    fn authorize_with_all<F: FnOnce() -> O, O>(&self, f: F) -> O;
}

pub trait ScryptoGenericBucket {
    fn as_fungible(&self) -> FungibleBucket;

    fn as_non_fungible(&self) -> NonFungibleBucket;
}

pub trait ScryptoFungibleBucket {
    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> FungibleProof;

    fn authorize_with_amount<A: Into<Decimal>, F: FnOnce() -> O, O>(&self, amount: A, f: F) -> O;
}

pub trait ScryptoNonFungibleBucket {
    fn non_fungible_local_ids(&self) -> IndexSet<NonFungibleLocalId>;

    fn contains_non_fungible(&self, id: &NonFungibleLocalId) -> bool;

    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;

    fn non_fungible_local_id(&self) -> NonFungibleLocalId;

    fn non_fungible_global_id(&self) -> NonFungibleGlobalId;

    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;

    fn take_non_fungibles(&mut self, non_fungible_local_ids: &IndexSet<NonFungibleLocalId>)
        -> Self;

    fn take_non_fungible(&mut self, non_fungible_local_id: &NonFungibleLocalId) -> Self;

    fn create_proof_of_non_fungibles(
        &self,
        non_fungible_local_ids: &IndexSet<NonFungibleLocalId>,
    ) -> NonFungibleProof;

    fn authorize_with_non_fungibles<F: FnOnce() -> O, O>(
        &self,
        non_fungible_local_ids: &IndexSet<NonFungibleLocalId>,
        f: F,
    ) -> O;
}

//=============
// Any bucket
//=============

impl ScryptoBucket for Bucket {
    type ProofType = Proof;
    type ResourceManagerType = ResourceManager;

    fn new(resource_address: ResourceAddress) -> Self {
        let rtn = ScryptoVmV1Api::object_call(
            resource_address.as_node_id(),
            RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT,
            scrypto_encode(&ResourceManagerCreateEmptyBucketInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn drop_empty(self) {
        let resource_address = self.resource_address();
        ScryptoVmV1Api::object_call(
            resource_address.as_node_id(),
            RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT,
            scrypto_encode(&ResourceManagerDropEmptyBucketInput {
                bucket: Bucket(self.0),
            })
            .unwrap(),
        );
    }

    fn burn(self) {
        let manager = self.resource_manager();
        manager.burn(self);
    }

    fn create_proof_of_all(&self) -> Self::ProofType {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            BUCKET_CREATE_PROOF_OF_ALL_IDENT,
            scrypto_encode(&BucketCreateProofOfAllInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
    }

    fn resource_address(&self) -> ResourceAddress {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            BUCKET_GET_RESOURCE_ADDRESS_IDENT,
            scrypto_encode(&BucketGetResourceAddressInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn put(&mut self, other: Self) -> () {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            BUCKET_PUT_IDENT,
            scrypto_encode(&BucketPutInput { bucket: other }).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn amount(&self) -> Decimal {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            BUCKET_GET_AMOUNT_IDENT,
            scrypto_encode(&BucketGetAmountInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    /// Takes some amount of resources from this bucket.
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            BUCKET_TAKE_IDENT,
            scrypto_encode(&BucketTakeInput {
                amount: amount.into(),
            })
            .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn take_advanced<A: Into<Decimal>>(
        &mut self,
        amount: A,
        withdraw_strategy: WithdrawStrategy,
    ) -> Self {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            BUCKET_TAKE_ADVANCED_IDENT,
            scrypto_encode(&BucketTakeAdvancedInput {
                amount: amount.into(),
                withdraw_strategy,
            })
            .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    /// Checks if this bucket is empty.
    fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    fn authorize_with_all<F: FnOnce() -> O, O>(&self, f: F) -> O {
        LocalAuthZone::push(self.create_proof_of_all());
        let output = f();
        LocalAuthZone::pop()
            .expect("Authorized closure changed auth zone proof stack")
            .drop();
        output
    }
}

impl ScryptoGenericBucket for Bucket {
    fn as_fungible(&self) -> FungibleBucket {
        assert!(
            self.resource_address()
                .as_node_id()
                .is_global_fungible_resource_manager(),
            "Not a fungible bucket"
        );
        FungibleBucket(Self(self.0))
    }

    fn as_non_fungible(&self) -> NonFungibleBucket {
        assert!(
            self.resource_address()
                .as_node_id()
                .is_global_non_fungible_resource_manager(),
            "Not a non-fungible bucket"
        );
        NonFungibleBucket(Self(self.0))
    }
}

//=================
// Fungible bucket
//=================

impl ScryptoBucket for FungibleBucket {
    type ProofType = FungibleProof;
    type ResourceManagerType = FungibleResourceManager;

    fn new(resource_address: ResourceAddress) -> Self {
        assert!(resource_address
            .as_node_id()
            .is_global_fungible_resource_manager());
        Self(Bucket::new(resource_address))
    }

    fn drop_empty(self) {
        self.0.drop_empty()
    }

    fn burn(self) {
        self.0.burn()
    }

    fn create_proof_of_all(&self) -> Self::ProofType {
        FungibleProof(self.0.create_proof_of_all())
    }

    fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
    }

    fn put(&mut self, other: Self) -> () {
        self.0.put(other.0)
    }

    fn amount(&self) -> Decimal {
        self.0.amount()
    }

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        Self(self.0.take(amount))
    }

    fn take_advanced<A: Into<Decimal>>(
        &mut self,
        amount: A,
        withdraw_strategy: WithdrawStrategy,
    ) -> Self {
        Self(self.0.take_advanced(amount, withdraw_strategy))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn authorize_with_all<F: FnOnce() -> O, O>(&self, f: F) -> O {
        self.0.authorize_with_all(f)
    }
}

impl ScryptoFungibleBucket for FungibleBucket {
    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> FungibleProof {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            FUNGIBLE_BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&FungibleBucketCreateProofOfAmountInput {
                amount: amount.into(),
            })
            .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn authorize_with_amount<A: Into<Decimal>, F: FnOnce() -> O, O>(&self, amount: A, f: F) -> O {
        LocalAuthZone::push(self.create_proof_of_amount(amount));
        let output = f();
        LocalAuthZone::pop()
            .expect("Authorized closure changed auth zone proof stack")
            .drop();
        output
    }
}

//====================
// Non-Fungible bucket
//====================

impl ScryptoBucket for NonFungibleBucket {
    type ProofType = NonFungibleProof;
    type ResourceManagerType = NonFungibleResourceManager;

    fn new(resource_address: ResourceAddress) -> Self {
        assert!(resource_address
            .as_node_id()
            .is_global_non_fungible_resource_manager());
        Self(Bucket::new(resource_address))
    }

    fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
    }

    fn drop_empty(self) {
        self.0.drop_empty()
    }

    fn burn(self) {
        self.0.burn()
    }

    fn create_proof_of_all(&self) -> Self::ProofType {
        NonFungibleProof(self.0.create_proof_of_all())
    }

    fn put(&mut self, other: Self) -> () {
        self.0.put(other.0)
    }

    fn amount(&self) -> Decimal {
        self.0.amount()
    }

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        Self(self.0.take(amount))
    }

    fn take_advanced<A: Into<Decimal>>(
        &mut self,
        amount: A,
        withdraw_strategy: WithdrawStrategy,
    ) -> Self {
        Self(self.0.take_advanced(amount, withdraw_strategy))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn authorize_with_all<F: FnOnce() -> O, O>(&self, f: F) -> O {
        self.0.authorize_with_all(f)
    }
}

impl ScryptoNonFungibleBucket for NonFungibleBucket {
    fn non_fungible_local_ids(&self) -> IndexSet<NonFungibleLocalId> {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
            scrypto_encode(&BucketGetNonFungibleLocalIdsInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn contains_non_fungible(&self, id: &NonFungibleLocalId) -> bool {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            NON_FUNGIBLE_BUCKET_CONTAINS_NON_FUNGIBLE_IDENT,
            scrypto_encode(&NonFungibleBucketContainsNonFungibleInput { id: id.clone() }).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket.
    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self.non_fungible_local_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleGlobalId::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns the non-fungible local id if this is a singleton non-fungible bucket.
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

    /// Returns the non-fungible global id if this is a singleton non-fungible bucket.
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    fn non_fungible_global_id(&self) -> NonFungibleGlobalId {
        NonFungibleGlobalId::new(self.resource_address(), self.non_fungible_local_id())
    }

    /// Returns a singleton non-fungible.
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT bucket");
        }
        non_fungibles.into_iter().next().unwrap()
    }

    /// Takes a specific non-fungible from this bucket.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    fn take_non_fungible(&mut self, non_fungible_local_id: &NonFungibleLocalId) -> Self {
        self.take_non_fungibles(&indexset!(non_fungible_local_id.clone()))
    }

    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &IndexSet<NonFungibleLocalId>,
    ) -> Self {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT,
            scrypto_encode(&BucketTakeNonFungiblesInput {
                ids: non_fungible_local_ids.clone(),
            })
            .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_non_fungibles(
        &self,
        ids: &IndexSet<NonFungibleLocalId>,
    ) -> NonFungibleProof {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleBucketCreateProofOfNonFungiblesInput { ids: ids.clone() })
                .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn authorize_with_non_fungibles<F: FnOnce() -> O, O>(
        &self,
        non_fungible_local_ids: &IndexSet<NonFungibleLocalId>,
        f: F,
    ) -> O {
        LocalAuthZone::push(self.create_proof_of_non_fungibles(non_fungible_local_ids));
        let output = f();
        LocalAuthZone::pop()
            .expect("Authorized closure changed auth zone proof stack")
            .drop();
        output
    }
}
