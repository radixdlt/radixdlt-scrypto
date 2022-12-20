use radix_engine_interface::api::api::Invokable;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use scrypto::engine::scrypto_env::ScryptoEnv;
use scrypto::scrypto_env_native_fn;

use crate::resource::*;
use crate::scrypto;

pub trait ScryptoVault {
    fn with_bucket(bucket: Bucket) -> Self;
    fn amount(&self) -> Decimal;
    fn new(resource_address: ResourceAddress) -> Self;
    fn take_internal(&mut self, amount: Decimal) -> Bucket;
    fn lock_fee_internal(&mut self, amount: Decimal) -> ();
    fn lock_contingent_fee_internal(&mut self, amount: Decimal) -> ();
    fn put(&mut self, bucket: Bucket) -> ();
    fn take_non_fungibles(&mut self, non_fungible_ids: &BTreeSet<NonFungibleId>) -> Bucket;
    fn resource_address(&self) -> ResourceAddress;
    fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId>;
    fn create_proof(&self) -> Proof;
    fn create_proof_by_amount(&self, amount: Decimal) -> Proof;
    fn create_proof_by_ids(&self, ids: &BTreeSet<NonFungibleId>) -> Proof;
    fn lock_fee<A: Into<Decimal>>(&mut self, amount: A);
    fn lock_contingent_fee<A: Into<Decimal>>(&mut self, amount: A);
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Bucket;
    fn take_all(&mut self) -> Bucket;
    fn take_non_fungible(&mut self, non_fungible_id: &NonFungibleId) -> Bucket;
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;
    fn is_empty(&self) -> bool;
    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;
    fn non_fungible_id(&self) -> NonFungibleId;
    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;
}

impl ScryptoVault for Vault {
    /// Creates an empty vault and fills it with an initial bucket of resource.
    fn with_bucket(bucket: Bucket) -> Self {
        let vault = Vault::new(bucket.resource_address());
        let mut vault = Vault(vault.0);
        vault.put(bucket);
        vault
    }

    fn amount(&self) -> Decimal {
        let mut env = ScryptoEnv;
        env.invoke(VaultGetAmountInvocation { receiver: self.0 })
            .unwrap()
    }

    scrypto_env_native_fn! {
        fn new(resource_address: ResourceAddress) -> Self {
            ResourceManagerCreateVaultInvocation {
                receiver: resource_address,
            }
        }

        fn take_internal(&mut self, amount: Decimal) -> Bucket {
            VaultTakeInvocation {
                receiver: self.0,
                amount,
            }
        }

        fn lock_fee_internal(&mut self, amount: Decimal) -> () {
            VaultLockFeeInvocation {
                receiver: self.0,
                amount,
                contingent: false,
            }
        }

        fn lock_contingent_fee_internal(&mut self, amount: Decimal) -> () {
            VaultLockFeeInvocation {
                receiver: self.0,
                amount,
                contingent: true,
            }
        }


        fn put(&mut self, bucket: Bucket) -> () {
            VaultPutInvocation {
                receiver: self.0,
                bucket: Bucket(bucket.0),
            }
        }

        fn take_non_fungibles(&mut self, non_fungible_ids: &BTreeSet<NonFungibleId>) -> Bucket {
            VaultTakeNonFungiblesInvocation {
                receiver: self.0,
                non_fungible_ids: non_fungible_ids.clone(),
            }
        }

        fn resource_address(&self) -> ResourceAddress {
            VaultGetResourceAddressInvocation {
                receiver: self.0,
            }
        }

        fn non_fungible_ids(&self) -> BTreeSet<NonFungibleId> {
            VaultGetNonFungibleIdsInvocation {
                receiver: self.0,
            }
        }

        fn create_proof(&self) -> Proof {
            VaultCreateProofInvocation {
                receiver: self.0,
            }
        }

        fn create_proof_by_amount(&self, amount: Decimal) -> Proof {
            VaultCreateProofByAmountInvocation { amount, receiver: self.0, }
        }

        fn create_proof_by_ids(&self, ids: &BTreeSet<NonFungibleId>) -> Proof {
            VaultCreateProofByIdsInvocation { ids: ids.clone(), receiver: self.0 }
        }
    }

    /// Locks the specified amount as transaction fee.
    ///
    /// Unused fee will be refunded to the vaults from the most recently locked to the least.
    fn lock_fee<A: Into<Decimal>>(&mut self, amount: A) {
        self.lock_fee_internal(amount.into())
    }

    /// Locks the given amount of resource as contingent fee.
    ///
    /// The locked amount will be used as transaction only if the transaction succeeds;
    /// Unused amount will be refunded the original vault.
    fn lock_contingent_fee<A: Into<Decimal>>(&mut self, amount: A) {
        self.lock_contingent_fee_internal(amount.into())
    }

    /// Takes some amount of resource from this vault into a bucket.
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Bucket {
        let bucket = self.take_internal(amount.into());
        Bucket(bucket.0)
    }

    /// Takes all resource stored in this vault.
    fn take_all(&mut self) -> Bucket {
        self.take(self.amount())
    }

    /// Takes a specific non-fungible from this vault.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault or the specified non-fungible resource is not found.
    fn take_non_fungible(&mut self, non_fungible_id: &NonFungibleId) -> Bucket {
        let bucket = self.take_non_fungibles(&BTreeSet::from([non_fungible_id.clone()]));
        Bucket(bucket.0)
    }

    /// Uses resources in this vault as authorization for an operation.
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        ComponentAuthZone::push(self.create_proof());
        let output = f();
        ComponentAuthZone::pop().drop();
        output
    }

    /// Checks if this vault is empty.
    fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault.
    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>> {
        let resource_address = self.resource_address();
        self.non_fungible_ids()
            .iter()
            .map(|id| NonFungible::from(NonFungibleAddress::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns a singleton non-fungible id
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    fn non_fungible_id(&self) -> NonFungibleId {
        let non_fungible_ids = self.non_fungible_ids();
        if non_fungible_ids.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        self.non_fungible_ids().into_iter().next().unwrap()
    }

    /// Returns a singleton non-fungible.
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        non_fungibles.into_iter().next().unwrap()
    }
}
