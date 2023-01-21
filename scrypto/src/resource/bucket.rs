use crate::resource::{ComponentAuthZone, NonFungible, ScryptoProof};
use radix_engine_interface::api::Invokable;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use scrypto::engine::scrypto_env::ScryptoEnv;
use scrypto::scrypto_env_native_fn;

pub trait ScryptoBucket {
    fn new(resource_address: ResourceAddress) -> Self;
    fn burn(self);
    fn create_proof(&self) -> Proof;
    fn resource_address(&self) -> ResourceAddress;
    fn take_internal(&mut self, amount: Decimal) -> Bucket;
    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Bucket;
    fn put(&mut self, other: Self) -> ();
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId>;
    fn amount(&self) -> Decimal;
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self;
    fn take_non_fungible(&mut self, non_fungible_local_id: &NonFungibleLocalId) -> Self;
    fn is_empty(&self) -> bool;
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;
    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;
    fn non_fungible_local_id(&self) -> NonFungibleLocalId;
    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;
}

impl ScryptoBucket for Bucket {
    fn new(resource_address: ResourceAddress) -> Self {
        let mut env = ScryptoEnv;
        env.invoke(ResourceManagerCreateBucketInvocation {
            receiver: resource_address,
        })
        .unwrap()
    }

    fn burn(self) {
        let mut env = ScryptoEnv;
        let receiver = self.resource_address();
        env.invoke(ResourceManagerBurnInvocation {
            receiver,
            bucket: Bucket(self.0),
        })
        .unwrap();
    }

    fn create_proof(&self) -> Proof {
        let mut env = ScryptoEnv;
        env.invoke(BucketCreateProofInvocation { receiver: self.0 })
            .unwrap()
    }

    fn resource_address(&self) -> ResourceAddress {
        let mut env = ScryptoEnv;
        env.invoke(BucketGetResourceAddressInvocation { receiver: self.0 })
            .unwrap()
    }

    scrypto_env_native_fn! {
        fn take_internal(&mut self, amount: Decimal) -> Bucket {
            BucketTakeInvocation {
                receiver: self.0,
                amount,
            }
        }

        fn take_non_fungibles(&mut self, non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>) -> Bucket {
            BucketTakeNonFungiblesInvocation {
                receiver: self.0,
                ids: non_fungible_local_ids.clone()
            }
        }

        fn put(&mut self, other: Self) -> () {
            BucketPutInvocation {
                receiver: self.0,
                bucket: Bucket(other.0),
            }
        }

        fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId> {
            BucketGetNonFungibleLocalIdsInvocation {
                receiver: self.0,
            }
        }

        fn amount(&self) -> Decimal {
            BucketGetAmountInvocation {
                receiver: self.0,
            }
        }
    }

    /// Takes some amount of resources from this bucket.
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self {
        self.take_internal(amount.into())
    }

    /// Takes a specific non-fungible from this bucket.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible bucket or the specified non-fungible resource is not found.
    fn take_non_fungible(&mut self, non_fungible_local_id: &NonFungibleLocalId) -> Self {
        self.take_non_fungibles(&BTreeSet::from([non_fungible_local_id.clone()]))
    }

    /// Uses resources in this bucket as authorization for an operation.
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        ComponentAuthZone::push(self.create_proof());
        let output = f();
        ComponentAuthZone::pop().drop();
        output
    }

    /// Checks if this bucket is empty.
    fn is_empty(&self) -> bool {
        self.amount() == 0.into()
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
}
