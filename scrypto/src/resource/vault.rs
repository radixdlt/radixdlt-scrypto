use crate::resource::*;
use crate::*;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use runtime::LocalAuthZone;
use sbor::rust::prelude::*;
use scrypto::engine::scrypto_env::ScryptoEnv;

//========
// Traits
//========

pub trait ScryptoVault {
    type BucketType;
    type ProofType;

    fn with_bucket(bucket: Self::BucketType) -> Self;

    fn new(resource_address: ResourceAddress) -> Self;

    fn put(&mut self, bucket: Self::BucketType) -> ();

    fn amount(&self) -> Decimal;

    fn resource_address(&self) -> ResourceAddress;

    fn resource_manager(&self) -> ResourceManager {
        self.resource_address().into()
    }

    fn is_empty(&self) -> bool;

    fn create_proof(&self) -> Self::ProofType;

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> Self::ProofType;

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self::BucketType;

    fn take_all(&mut self) -> Self::BucketType;

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O;

    fn as_fungible(&self) -> FungibleVault;

    fn as_non_fungible(&self) -> NonFungibleVault;

    fn burn<A: Into<Decimal>>(&mut self, amount: A);
}

pub trait ScryptoFungibleVault {
    fn lock_fee<A: Into<Decimal>>(&mut self, amount: A);

    fn lock_contingent_fee<A: Into<Decimal>>(&mut self, amount: A);
}

pub trait ScryptoNonFungibleVault {
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId>;

    fn non_fungibles<T: NonFungibleData>(&self) -> Vec<NonFungible<T>>;

    fn non_fungible_local_id(&self) -> NonFungibleLocalId;

    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T>;

    fn take_non_fungible(
        &mut self,
        non_fungible_local_id: &NonFungibleLocalId,
    ) -> NonFungibleBucket;

    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> NonFungibleBucket;

    fn create_proof_of_non_fungibles(&self, ids: BTreeSet<NonFungibleLocalId>) -> NonFungibleProof;

    fn burn_non_fungibles(&mut self, ids: &BTreeSet<NonFungibleLocalId>);
}

//===========
// Any vault
//===========

impl ScryptoVault for Vault {
    type BucketType = Bucket;

    type ProofType = Proof;

    /// Creates an empty vault and fills it with an initial bucket of resource.
    fn with_bucket(bucket: Bucket) -> Self {
        let mut vault = Vault::new(bucket.resource_address());
        vault.put(bucket);
        vault
    }

    fn new(resource_address: ResourceAddress) -> Self {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                resource_address.as_node_id(),
                RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT,
                scrypto_encode(&ResourceManagerCreateEmptyVaultInput {}).unwrap(),
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

    fn resource_address(&self) -> ResourceAddress {
        let mut env = ScryptoEnv;
        let info = env.get_object_info(self.0.as_node_id()).unwrap();
        ResourceAddress::try_from(info.get_outer_object().as_ref()).unwrap()
    }

    fn create_proof(&self) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                VAULT_CREATE_PROOF_IDENT,
                scrypto_encode(&VaultCreateProofInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> Proof {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                VAULT_CREATE_PROOF_OF_AMOUNT_IDENT,
                scrypto_encode(&VaultCreateProofOfAmountInput {
                    amount: amount.into(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    /// Takes some amount of resource from this vault into a bucket.
    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Bucket {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                VAULT_TAKE_IDENT,
                scrypto_encode(&VaultTakeInput {
                    amount: amount.into(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    /// Takes all resource stored in this vault.
    fn take_all(&mut self) -> Bucket {
        self.take(self.amount())
    }

    /// Uses resources in this vault as authorization for an operation.
    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        LocalAuthZone::push(self.create_proof());
        let output = f();
        LocalAuthZone::pop().drop();
        output
    }

    /// Checks if this vault is empty.
    fn is_empty(&self) -> bool {
        self.amount() == 0.into()
    }

    fn as_fungible(&self) -> FungibleVault {
        assert!(self.0.as_node_id().is_internal_fungible_vault());
        FungibleVault(Vault(self.0))
    }

    fn as_non_fungible(&self) -> NonFungibleVault {
        assert!(self.0.as_node_id().is_internal_non_fungible_vault());
        NonFungibleVault(Vault(self.0))
    }

    fn burn<A: Into<Decimal>>(&mut self, amount: A) {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                VAULT_BURN_IDENT,
                scrypto_encode(&VaultBurnInput {
                    amount: amount.into(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}

//================
// Fungible vault
//================

impl ScryptoVault for FungibleVault {
    type BucketType = FungibleBucket;

    type ProofType = FungibleProof;

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

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn create_proof(&self) -> Self::ProofType {
        FungibleProof(self.0.create_proof())
    }

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> Self::ProofType {
        FungibleProof(self.0.create_proof_of_amount(amount))
    }

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self::BucketType {
        FungibleBucket(self.0.take(amount))
    }

    fn take_all(&mut self) -> Self::BucketType {
        FungibleBucket(self.0.take_all())
    }

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        self.0.authorize(f)
    }

    fn as_fungible(&self) -> FungibleVault {
        self.0.as_fungible()
    }

    fn as_non_fungible(&self) -> NonFungibleVault {
        self.0.as_non_fungible()
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
        let mut env = ScryptoEnv;
        let _rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                FUNGIBLE_VAULT_LOCK_FEE_IDENT,
                scrypto_encode(&FungibleVaultLockFeeInput {
                    amount: amount.into(),
                    contingent: false,
                })
                .unwrap(),
            )
            .unwrap();
    }

    /// Locks the given amount of resource as contingent fee.
    ///
    /// The locked amount will be used as transaction only if the transaction succeeds;
    /// Unused amount will be refunded the original vault.
    fn lock_contingent_fee<A: Into<Decimal>>(&mut self, amount: A) {
        let mut env = ScryptoEnv;
        let _rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                FUNGIBLE_VAULT_LOCK_FEE_IDENT,
                scrypto_encode(&FungibleVaultLockFeeInput {
                    amount: amount.into(),
                    contingent: true,
                })
                .unwrap(),
            )
            .unwrap();
    }
}

//====================
// Non-fungible vault
//====================

impl ScryptoVault for NonFungibleVault {
    type BucketType = NonFungibleBucket;

    type ProofType = NonFungibleProof;

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

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn create_proof(&self) -> Self::ProofType {
        NonFungibleProof(self.0.create_proof())
    }

    fn create_proof_of_amount<A: Into<Decimal>>(&self, amount: A) -> Self::ProofType {
        NonFungibleProof(self.0.create_proof_of_amount(amount))
    }

    fn take<A: Into<Decimal>>(&mut self, amount: A) -> Self::BucketType {
        NonFungibleBucket(self.0.take(amount))
    }

    fn take_all(&mut self) -> Self::BucketType {
        NonFungibleBucket(self.0.take_all())
    }

    fn authorize<F: FnOnce() -> O, O>(&self, f: F) -> O {
        self.0.authorize(f)
    }

    fn as_fungible(&self) -> FungibleVault {
        self.0.as_fungible()
    }

    fn as_non_fungible(&self) -> NonFungibleVault {
        self.0.as_non_fungible()
    }

    fn burn<A: Into<Decimal>>(&mut self, amount: A) {
        self.0.burn(amount)
    }
}

impl ScryptoNonFungibleVault for NonFungibleVault {
    fn non_fungible_local_ids(&self) -> BTreeSet<NonFungibleLocalId> {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                scrypto_encode(&NonFungibleVaultGetNonFungibleLocalIdsInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    /// Returns all the non-fungible units contained.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible vault.
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
    /// Panics if this is not a singleton bucket
    fn non_fungible<T: NonFungibleData>(&self) -> NonFungible<T> {
        let non_fungibles = self.non_fungibles();
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
        self.take_non_fungibles(&BTreeSet::from([non_fungible_local_id.clone()]))
    }

    fn take_non_fungibles(
        &mut self,
        non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>,
    ) -> NonFungibleBucket {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT,
                scrypto_encode(&NonFungibleVaultTakeNonFungiblesInput {
                    non_fungible_local_ids: non_fungible_local_ids.clone(),
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
                NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
                scrypto_encode(&NonFungibleVaultCreateProofOfNonFungiblesInput { ids }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    fn burn_non_fungibles(&mut self, non_fungible_local_ids: &BTreeSet<NonFungibleLocalId>) {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0 .0.as_node_id(),
                NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT,
                scrypto_encode(&NonFungibleVaultBurnNonFungiblesInput {
                    non_fungible_local_ids: non_fungible_local_ids.clone(),
                })
                .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}
