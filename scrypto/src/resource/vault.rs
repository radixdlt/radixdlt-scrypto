use crate::resource::*;
use crate::*;
use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_common::traits::NonFungibleData;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use runtime::LocalAuthZone;
use sbor::rust::prelude::*;
use scrypto::engine::scrypto_env::ScryptoVmV1Api;

//========
// Traits
//========

pub trait ScryptoVault {
    type BucketType;
    type ResourceManagerType;

    fn with_bucket(bucket: Self::BucketType) -> Self;

    fn new(resource_address: ResourceAddress) -> Self;

    fn put(&mut self, bucket: Self::BucketType) -> ();

    fn amount(&self) -> Decimal;

    fn resource_address(&self) -> ResourceAddress;

    fn resource_manager(&self) -> Self::ResourceManagerType;

    fn is_empty(&self) -> bool;

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self::BucketType;

    fn take_all(&mut self) -> Self::BucketType;

    fn take_advanced<A: Into<Decimal>>(
        &mut self,
        amount: A,
        withdraw_strategy: WithdrawStrategy,
    ) -> Self::BucketType;

    fn burn<A: Into<Decimal>>(&mut self, amount: A);
}

pub trait ScryptoGenericVault {
    fn as_fungible(&self) -> FungibleVault;

    fn as_non_fungible(&self) -> NonFungibleVault;
}

pub trait ScryptoFungibleVault {
    fn lock_fee<A: Into<Decimal>>(&mut self, amount: A);

    fn lock_contingent_fee<A: Into<Decimal>>(&mut self, amount: A);

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> FungibleProof;

    fn authorize_with_amount<A: Into<Decimal>, F: FnOnce() -> O, O>(&self, amount: A, f: F) -> O;
}

pub trait ScryptoNonFungibleVault {
    fn non_fungible_local_ids(&self, limit: u32) -> IndexSet<NonFungibleLocalId>;

    fn contains_non_fungible(&self, id: &NonFungibleLocalId) -> bool;

    fn non_fungibles<T: NonFungibleData>(&self, limit: u32) -> Vec<NonFungible<T>>;

    fn non_fungible_local_id(&self) -> NonFungibleLocalId;

    fn non_fungible_global_id(&self) -> NonFungibleGlobalId;

    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;

    fn take_non_fungible(
        &mut self,
        non_fungible_local_id: &NonFungibleLocalId,
    ) -> NonFungibleBucket;

    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &IndexSet<NonFungibleLocalId>,
    ) -> NonFungibleBucket;

    fn burn_non_fungibles(&mut self, non_fungible_local_ids: &IndexSet<NonFungibleLocalId>);

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

//===========
// Any vault
//===========

impl ScryptoVault for Vault {
    type BucketType = Bucket;
    type ResourceManagerType = ResourceManager;

    /// Creates an empty vault and fills it with an initial bucket of resource.
    fn with_bucket(bucket: Self::BucketType) -> Self {
        let mut vault = Vault::new(bucket.resource_address());
        vault.put(bucket);
        vault
    }

    fn new(resource_address: ResourceAddress) -> Self {
        let rtn = ScryptoVmV1Api::object_call(
            resource_address.as_node_id(),
            RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT,
            scrypto_encode(&ResourceManagerCreateEmptyVaultInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn put(&mut self, bucket: Self::BucketType) -> () {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            VAULT_PUT_IDENT,
            scrypto_encode(&VaultPutInput { bucket }).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn amount(&self) -> Decimal {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            VAULT_GET_AMOUNT_IDENT,
            scrypto_encode(&VaultGetAmountInput {}).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn resource_address(&self) -> ResourceAddress {
        let address = ScryptoVmV1Api::object_get_outer_object(self.0.as_node_id());
        ResourceAddress::try_from(address).unwrap()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
    }

    /// Takes some amount of resource from this vault into a bucket.
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self::BucketType {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            VAULT_TAKE_IDENT,
            scrypto_encode(&VaultTakeInput {
                amount: amount.into(),
            })
            .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    /// Takes all resource stored in this vault.
    fn take_all(&mut self) -> Self::BucketType {
        self.take(self.amount())
    }

    fn take_advanced<A: Into<Decimal>>(
        &mut self,
        amount: A,
        withdraw_strategy: WithdrawStrategy,
    ) -> Self::BucketType {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            VAULT_TAKE_ADVANCED_IDENT,
            scrypto_encode(&VaultTakeAdvancedInput {
                amount: amount.into(),
                withdraw_strategy,
            })
            .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    /// Checks if this vault is empty.
    fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    fn burn<A: Into<Decimal>>(&mut self, amount: A) {
        let rtn = ScryptoVmV1Api::object_call(
            self.0.as_node_id(),
            VAULT_BURN_IDENT,
            scrypto_encode(&VaultBurnInput {
                amount: amount.into(),
            })
            .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }
}

impl ScryptoGenericVault for Vault {
    fn as_fungible(&self) -> FungibleVault {
        assert!(
            self.0.as_node_id().is_internal_fungible_vault(),
            "Not a fungible vault"
        );
        FungibleVault(Self(self.0))
    }

    fn as_non_fungible(&self) -> NonFungibleVault {
        assert!(
            self.0.as_node_id().is_internal_non_fungible_vault(),
            "Not a non-fungible vault"
        );
        NonFungibleVault(Self(self.0))
    }
}

//================
// Fungible vault
//================

impl ScryptoVault for FungibleVault {
    type BucketType = FungibleBucket;
    type ResourceManagerType = FungibleResourceManager;

    fn with_bucket(bucket: Self::BucketType) -> Self {
        Self(Vault::with_bucket(bucket.0))
    }

    fn new(resource_address: ResourceAddress) -> Self {
        assert!(resource_address
            .as_node_id()
            .is_global_fungible_resource_manager());
        Self(Vault::new(resource_address))
    }

    fn put(&mut self, bucket: Self::BucketType) -> () {
        self.0.put(bucket.0)
    }

    fn amount(&self) -> Decimal {
        self.0.amount()
    }

    fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self::BucketType {
        FungibleBucket(self.0.take(amount))
    }

    fn take_all(&mut self) -> Self::BucketType {
        FungibleBucket(self.0.take_all())
    }

    fn take_advanced<A: Into<Decimal>>(
        &mut self,
        amount: A,
        withdraw_strategy: WithdrawStrategy,
    ) -> Self::BucketType {
        FungibleBucket(self.0.take_advanced(amount, withdraw_strategy))
    }

    fn burn<A: Into<Decimal>>(&mut self, amount: A) {
        self.0.burn(amount)
    }
}

impl ScryptoFungibleVault for FungibleVault {
    /// Locks the specified amount as transaction fee.
    ///
    /// Unused fee will be refunded to the vaults from the most recently locked to the least.
    fn lock_fee<A: Into<Decimal>>(&mut self, amount: A) {
        let _rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            FUNGIBLE_VAULT_LOCK_FEE_IDENT,
            scrypto_encode(&FungibleVaultLockFeeInput {
                amount: amount.into(),
                contingent: false,
            })
            .unwrap(),
        );
    }

    /// Locks the given amount of resource as contingent fee.
    ///
    /// The locked amount will be used as transaction only if the transaction succeeds;
    /// Unused amount will be refunded the original vault.
    fn lock_contingent_fee<A: Into<Decimal>>(&mut self, amount: A) {
        let _rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            FUNGIBLE_VAULT_LOCK_FEE_IDENT,
            scrypto_encode(&FungibleVaultLockFeeInput {
                amount: amount.into(),
                contingent: true,
            })
            .unwrap(),
        );
    }

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> FungibleProof {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_IDENT,
            scrypto_encode(&FungibleVaultCreateProofOfAmountInput {
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
// Non-fungible vault
//====================

impl ScryptoVault for NonFungibleVault {
    type BucketType = NonFungibleBucket;
    type ResourceManagerType = NonFungibleResourceManager;

    fn with_bucket(bucket: Self::BucketType) -> Self {
        Self(Vault::with_bucket(bucket.0))
    }

    fn new(resource_address: ResourceAddress) -> Self {
        assert!(resource_address
            .as_node_id()
            .is_global_non_fungible_resource_manager());
        Self(Vault::new(resource_address))
    }

    fn put(&mut self, bucket: Self::BucketType) -> () {
        self.0.put(bucket.0)
    }

    fn amount(&self) -> Decimal {
        self.0.amount()
    }

    fn resource_address(&self) -> ResourceAddress {
        self.0.resource_address()
    }

    fn resource_manager(&self) -> Self::ResourceManagerType {
        self.resource_address().into()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self::BucketType {
        NonFungibleBucket(self.0.take(amount))
    }

    fn take_all(&mut self) -> Self::BucketType {
        NonFungibleBucket(self.0.take_all())
    }

    fn take_advanced<A: Into<Decimal>>(
        &mut self,
        amount: A,
        withdraw_strategy: WithdrawStrategy,
    ) -> Self::BucketType {
        NonFungibleBucket(self.0.take_advanced(amount, withdraw_strategy))
    }

    fn burn<A: Into<Decimal>>(&mut self, amount: A) {
        self.0.burn(amount)
    }
}

impl ScryptoNonFungibleVault for NonFungibleVault {
    fn non_fungible_local_ids(&self, limit: u32) -> IndexSet<NonFungibleLocalId> {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
            scrypto_encode(&NonFungibleVaultGetNonFungibleLocalIdsInput { limit }).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn contains_non_fungible(&self, id: &NonFungibleLocalId) -> bool {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            NON_FUNGIBLE_VAULT_CONTAINS_NON_FUNGIBLE_IDENT,
            scrypto_encode(&NonFungibleVaultContainsNonFungibleInput { id: id.clone() }).unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault.
    fn non_fungibles<T: NonFungibleData>(&self, limit: u32) -> Vec<NonFungible<T>> {
        let resource_address = self.0.resource_address();
        self.non_fungible_local_ids(limit)
            .iter()
            .map(|id| NonFungible::from(NonFungibleGlobalId::new(resource_address, id.clone())))
            .collect()
    }

    /// Returns the non-fungible local id if this is a singleton non-fungible vault.
    ///
    /// # Panics
    /// Panics if this is not a singleton vault
    fn non_fungible_local_id(&self) -> NonFungibleLocalId {
        let non_fungible_local_ids = self.non_fungible_local_ids(2);
        if non_fungible_local_ids.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        non_fungible_local_ids.into_iter().next().unwrap()
    }

    /// Returns the non-fungible global id if this is a singleton non-fungible vault.
    ///
    /// # Panics
    /// Panics if this is not a singleton vault
    fn non_fungible_global_id(&self) -> NonFungibleGlobalId {
        NonFungibleGlobalId::new(self.resource_address(), self.non_fungible_local_id())
    }

    /// Returns a singleton non-fungible.
    ///
    /// # Panics
    /// Panics if this is not a singleton bucket
    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        // Use limit of 2 in order to verify singleton
        let non_fungibles = self.non_fungibles(2);
        if non_fungibles.len() != 1 {
            panic!("Expecting singleton NFT vault");
        }
        non_fungibles.into_iter().next().unwrap()
    }

    /// Takes a specific non-fungible from this vault.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault or the specified non-fungible resource is not found.
    fn take_non_fungible(
        &mut self,
        non_fungible_local_id: &NonFungibleLocalId,
    ) -> NonFungibleBucket {
        self.take_non_fungibles(&indexset!(non_fungible_local_id.clone()))
    }

    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &IndexSet<NonFungibleLocalId>,
    ) -> NonFungibleBucket {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleVaultTakeNonFungiblesInput {
                non_fungible_local_ids: non_fungible_local_ids.clone(),
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
            NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleVaultCreateProofOfNonFungiblesInput { ids: ids.clone() })
                .unwrap(),
        );
        scrypto_decode(&rtn).unwrap()
    }

    fn burn_non_fungibles(&mut self, non_fungible_local_ids: &IndexSet<NonFungibleLocalId>) {
        let rtn = ScryptoVmV1Api::object_call(
            self.0 .0.as_node_id(),
            NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT,
            scrypto_encode(&NonFungibleVaultBurnNonFungiblesInput {
                non_fungible_local_ids: non_fungible_local_ids.clone(),
            })
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
