use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use scrypto::engine::scrypto_env::ScryptoEnv;

use crate::resource::*;
use crate::*;

pub trait ScryptoVault {
    fn with_bucket(bucket: Bucket) -> Self;
    fn amount(&self) -> Decimal;
    fn new(resource_address: ResourceAddress) -> Self;
    fn take_internal(&mut self, amount: Decimal) -> Bucket;
    fn lock_fee_internal(&mut self, amount: Decimal) -> ();
    fn lock_contingent_fee_internal(&mut self, amount: Decimal) -> ();
    fn put(&mut self, bucket: Bucket) -> ();
    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Bucket;
    fn resource_address(&self) -> ResourceAddress;
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId>;
    fn create_proof(&self) -> Proof;
    fn create_proof_by_amount(&self, amount: Decimal) -> Proof;
    fn create_proof_by_ids(&self, ids: &BTreeSet<NonFungibleLocalId>) -> Proof;
    fn lock_fee<A: Into<Decimal>>(&mut self, amount: A);
    fn lock_contingent_fee<A: Into<Decimal>>(&mut self, amount: A);
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Bucket;
    fn take_all(&mut self) -> Bucket;
    fn take_non_fungible(&mut self, non_fungible_local_id: &NonFungibleLocalId) -> Bucket;
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;
    fn is_empty(&self) -> bool;
    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;
    fn non_fungible_local_id(&self) -> NonFungibleLocalId;
    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;
}

impl ScryptoVault for Vault {
    /// Creates an empty vault and fills it with an initial bucket of resource.
    fn with_bucket(bucket: Bucket) -> Self {
        let mut vault = Vault::new(bucket.resource_address());
        vault.put(bucket);
        vault
    }

    fn amount(&self) -> Decimal {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                VAULT_GET_AMOUNT_IDENT,
                scrypto_encode(&VaultGetAmountInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn new(resource_address: ResourceAddress) -> Self {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                resource_address.as_node_id(),
                RESOURCE_MANAGER_CREATE_VAULT_IDENT,
                scrypto_encode(&ResourceManagerCreateVaultInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn take_internal(&mut self, amount: Decimal) -> Bucket {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                VAULT_TAKE_IDENT,
                scrypto_encode(&VaultTakeInput { amount }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn lock_fee_internal(&mut self, amount: Decimal) {
        let mut env = ScryptoEnv;
        let _rtn = env
            .call_method(
                self.0.as_node_id(),
                FUNGIBLE_VAULT_LOCK_FEE_IDENT,
                scrypto_encode(&FungibleVaultLockFeeInput {
                    amount,
                    contingent: false,
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn lock_contingent_fee_internal(&mut self, amount: Decimal) {
        let mut env = ScryptoEnv;
        let _rtn = env
            .call_method(
                self.0.as_node_id(),
                FUNGIBLE_VAULT_LOCK_FEE_IDENT,
                scrypto_encode(&FungibleVaultLockFeeInput {
                    amount,
                    contingent: true,
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Bucket {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT,
                scrypto_encode(&NonFungibleVaultTakeNonFungiblesInput {
                    non_fungible_local_ids: non_fungible_local_ids.clone(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn put(&mut self, bucket: Bucket) -> () {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                VAULT_PUT_IDENT,
                scrypto_encode(&VaultPutInput { bucket }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn resource_address(&self) -> ResourceAddress {
        let mut env = ScryptoEnv;
        let info = env.get_object_info(self.0.as_node_id()).unwrap();
        ResourceAddress::try_from(info.type_parent.unwrap().as_ref()).unwrap()
    }

    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId> {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                scrypto_encode(&NonFungibleVaultGetNonFungibleLocalIdsInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof(&self) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                VAULT_CREATE_PROOF_OF_ALL_IDENT,
                scrypto_encode(&VaultCreateProofOfAllInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_by_amount(&self, amount: Decimal) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                VAULT_CREATE_PROOF_OF_AMOUNT_IDENT,
                scrypto_encode(&VaultCreateProofOfAmountInput { amount }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_by_ids(&self, ids: &BTreeSet<NonFungibleLocalId>) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
                scrypto_encode(&NonFungibleVaultCreateProofOfNonFungiblesInput {
                    ids: ids.clone(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
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
    fn take_non_fungible(&mut self, non_fungible_local_id: &NonFungibleLocalId) -> Bucket {
        let bucket = self.take_non_fungibles(&BTreeSet::from([non_fungible_local_id.clone()]));
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
            panic!("Expecting singleton NFT vault");
        }
        non_fungibles.into_iter().next().unwrap()
    }
}
