use super::ScryptoUncheckedProof;
use crate::prelude::ResourceManager;
use crate::resource::NonFungible;
use crate::runtime::LocalAuthZone;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::NonFungibleData;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::prelude::*;
use scrypto::engine::scrypto_env::ScryptoEnv;

//=============
// Traits
//=============

pub trait ScryptoBucket {
    type ProofType;

    fn new(resource_address: ResourceAddress) -> Self;

    fn drop_empty(self);

    fn burn(self);

    fn create_proof(&self) -> Self::ProofType;

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> Self::ProofType;

    fn create_proof_of_all(&self) -> Self::ProofType;

    fn resource_address(&self) -> ResourceAddress;

    fn resource_manager(&self) -> ResourceManager {
        self.resource_address().into()
    }

    fn put(&mut self, other: Self) -> ();

    fn amount(&self) -> Decimal;

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self;

    fn is_empty(&self) -> bool;

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;

    fn as_fungible(&self) -> FungibleBucket;

    fn as_non_fungible(&self) -> NonFungibleBucket;
}

pub trait ScryptoFungibleBucket {}

pub trait ScryptoNonFungibleBucket {
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId>;

    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;

    fn non_fungible_local_id(&self) -> NonFungibleLocalId;

    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;

    fn take_non_fungibles(&mut self, non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>)
        -> Self;

    fn take_non_fungible(&mut self, non_fungible_local_id: &NonFungibleLocalId) -> Self;

    fn create_proof_of_non_fungibles(&self, ids: BTreeSet<NonFungibleLocalId>) -> NonFungibleProof;
}

//=============
// Any bucket
//=============

impl ScryptoBucket for Bucket {
    type ProofType = Proof;

    fn new(resource_address: ResourceAddress) -> Self {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                resource_address.as_node_id(),
                RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT,
                scrypto_encode(&ResourceManagerCreateEmptyBucketInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn drop_empty(self) {
        let resource_address = self.resource_address();
        ScryptoEnv
            .call_method(
                resource_address.as_node_id(),
                RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT,
                scrypto_encode(&ResourceManagerDropEmptyBucketInput {
                    bucket: Bucket(self.0),
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn burn(self) {
        let manager = self.resource_manager();
        manager.burn(self);
    }

    fn create_proof(&self) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                BUCKET_CREATE_PROOF_IDENT,
                scrypto_encode(&BucketCreateProofInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                BUCKET_CREATE_PROOF_OF_AMOUNT_IDENT,
                scrypto_encode(&BucketCreateProofOfAmountInput {
                    amount: amount.into(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_all(&self) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                BUCKET_CREATE_PROOF_OF_ALL_IDENT,
                scrypto_encode(&BucketCreateProofOfAllInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn resource_manager(&self) -> ResourceManager {
        self.resource_address().into()
    }

    fn resource_address(&self) -> ResourceAddress {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                BUCKET_GET_RESOURCE_ADDRESS_IDENT,
                scrypto_encode(&BucketGetResourceAddressInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn put(&mut self, other: Self) -> () {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                BUCKET_PUT_IDENT,
                scrypto_encode(&BucketPutInput { bucket: other }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn amount(&self) -> Decimal {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                BUCKET_GET_AMOUNT_IDENT,
                scrypto_encode(&BucketGetAmountInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    /// Takes some amount of resources from this bucket.
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                BUCKET_TAKE_IDENT,
                scrypto_encode(&BucketTakeInput {
                    amount: amount.into(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    /// Uses resources in this bucket as authorization for an operation.
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        LocalAuthZone::push(self.create_proof());
        let output = f();
        LocalAuthZone::pop().drop();
        output
    }

    /// Checks if this bucket is empty.
    fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    fn as_fungible(&self) -> FungibleBucket {
        assert!(self
            .resource_address()
            .as_node_id()
            .is_global_fungible_resource_manager());
        FungibleBucket(Bucket(self.0))
    }

    fn as_non_fungible(&self) -> NonFungibleBucket {
        assert!(self
            .resource_address()
            .as_node_id()
            .is_global_non_fungible_resource_manager());
        NonFungibleBucket(Bucket(self.0))
    }
}

//=================
// Fungible bucket
//=================

impl ScryptoBucket for FungibleBucket {
    type ProofType = FungibleProof;

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

    fn create_proof(&self) -> Self::ProofType {
        FungibleProof(self.0.create_proof())
    }

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> Self::ProofType {
        FungibleProof(self.0.create_proof_of_amount(amount))
    }

    fn create_proof_of_all(&self) -> Self::ProofType {
        FungibleProof(self.0.create_proof_of_all())
    }

    fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
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

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        self.0.authorize(f)
    }

    fn as_fungible(&self) -> FungibleBucket {
        self.0.as_fungible()
    }

    fn as_non_fungible(&self) -> NonFungibleBucket {
        self.0.as_non_fungible()
    }
}

impl ScryptoFungibleBucket for FungibleBucket {}

//====================
// Non-Fungible bucket
//====================

impl ScryptoBucket for NonFungibleBucket {
    type ProofType = NonFungibleProof;

    fn new(resource_address: ResourceAddress) -> Self {
        assert!(resource_address
            .as_node_id()
            .is_global_non_fungible_resource_manager());
        Self(Bucket::new(resource_address))
    }

    fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
    }

    fn drop_empty(self) {
        self.0.drop_empty()
    }

    fn burn(self) {
        self.0.burn()
    }

    fn create_proof(&self) -> Self::ProofType {
        NonFungibleProof(self.0.create_proof())
    }

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> Self::ProofType {
        NonFungibleProof(self.0.create_proof_of_amount(amount))
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

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        self.0.authorize(f)
    }

    fn as_fungible(&self) -> FungibleBucket {
        self.0.as_fungible()
    }

    fn as_non_fungible(&self) -> NonFungibleBucket {
        self.0.as_non_fungible()
    }
}

impl ScryptoNonFungibleBucket for NonFungibleBucket {
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId> {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                NON_FUNGIBLE_BUCKET_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                scrypto_encode(&BucketGetNonFungibleLocalIdsInput {}).unwrap(),
            )
            .unwrap();
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
        self.take_non_fungibles(&BTreeSet::from([non_fungible_local_id.clone()]))
    }

    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Self {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT,
                scrypto_encode(&BucketTakeNonFungiblesInput {
                    ids: non_fungible_local_ids.clone(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_non_fungibles(&self, ids: BTreeSet<NonFungibleLocalId>) -> NonFungibleProof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
                scrypto_encode(&NonFungibleBucketCreateProofOfNonFungiblesInput { ids }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}
