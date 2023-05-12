use crate::borrow_resource_manager;
use crate::resource::NonFungible;
use crate::runtime::LocalAuthZone;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::{model::*, ScryptoDecode, ScryptoEncode};
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::NonFungibleData;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use scrypto::engine::scrypto_env::ScryptoEnv;

use super::ScryptoUncheckedProof;

pub trait ScryptoBucket {
    fn new(resource_address: ResourceAddress) -> Self;

    fn drop_empty(self);

    fn burn(self);

    fn create_proof(&self) -> Proof;

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> Proof;

    fn create_proof_of_all(&self) -> Proof;

    fn resource_address(&self) -> ResourceAddress;

    fn put(&mut self, other: Self) -> ();

    fn amount(&self) -> Decimal;

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self;

    fn is_empty(&self) -> bool;

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;

    fn as_fungible_bucket(&self) -> FungibleBucket;

    fn as_no_fungible_bucket(&self) -> NonFungibleBucket;
}

pub trait ScryptoFungibleBucket {}

pub trait ScryptoNonFungibleBucket {
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId>;

    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;

    fn non_fungible_local_id(&self) -> NonFungibleLocalId;

    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;

    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Bucket;

    fn take_non_fungible(&mut self, non_fungible_local_id: &NonFungibleLocalId) -> Self;

    fn create_proof_of_non_fungibles(&self, ids: BTreeSet<NonFungibleLocalId>) -> Proof;
}

impl<T: AsRef<Bucket> + ScryptoEncode + ScryptoDecode> ScryptoBucket for T {
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
                    bucket: Bucket(self.as_ref().0),
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn burn(self) {
        let resource_address = self.resource_address();
        borrow_resource_manager!(resource_address).burn(Bucket(self.as_ref().0));
    }

    fn create_proof(&self) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.as_ref().0.as_node_id(),
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
                self.as_ref().0.as_node_id(),
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
                self.as_ref().0.as_node_id(),
                BUCKET_CREATE_PROOF_OF_ALL_IDENT,
                scrypto_encode(&BucketCreateProofOfAllInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn resource_address(&self) -> ResourceAddress {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.as_ref().0.as_node_id(),
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
                self.as_ref().0.as_node_id(),
                BUCKET_PUT_IDENT,
                scrypto_encode(&BucketPutInput {
                    bucket: Bucket(other.as_ref().0),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn amount(&self) -> Decimal {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.as_ref().0.as_node_id(),
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
                self.as_ref().0.as_node_id(),
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

    // TODO: should we check fungibility here?
    // Currently, it will fail at runtime when invoking fungible/non-fungible methods

    fn as_fungible_bucket(&self) -> FungibleBucket {
        FungibleBucket(Bucket(self.as_ref().0))
    }

    fn as_no_fungible_bucket(&self) -> NonFungibleBucket {
        NonFungibleBucket(Bucket(self.as_ref().0))
    }
}

impl ScryptoFungibleBucket for Bucket {}

impl ScryptoNonFungibleBucket for Bucket {
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId> {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
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
    ) -> Bucket {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                NON_FUNGIBLE_BUCKET_TAKE_NON_FUNGIBLES_IDENT,
                scrypto_encode(&BucketTakeNonFungiblesInput {
                    ids: non_fungible_local_ids.clone(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_non_fungibles(&self, ids: BTreeSet<NonFungibleLocalId>) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                NON_FUNGIBLE_BUCKET_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
                scrypto_encode(&NonFungibleBucketCreateProofOfNonFungiblesInput { ids }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}
