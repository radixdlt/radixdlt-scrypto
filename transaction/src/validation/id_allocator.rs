use radix_engine_interface::api::types::{
    AuthZoneStackId, BucketId, ComponentId, FeeReserveId, KeyValueStoreId, NonFungibleStoreId,
    PackageId, ProofId, ResourceManagerId, TransactionRuntimeId, VaultId,
};
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::model::*;

use sbor::rust::ops::Range;

use crate::errors::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdSpace {
    System,
    Transaction,
    Application,
}

/// An ID allocator defines how identities are generated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdAllocator {
    available: Range<u32>,
    transaction_hash: Hash,
}

impl IdAllocator {
    /// Creates an ID allocator.
    pub fn new(kind: IdSpace, transaction_hash: Hash) -> Self {
        Self {
            available: match kind {
                IdSpace::System => 0..512,
                IdSpace::Transaction => 512..1024,
                IdSpace::Application => 1024..u32::MAX,
            },
            transaction_hash,
        }
    }

    fn next(&mut self) -> Result<u32, IdAllocationError> {
        if self.available.len() > 0 {
            let id = self.available.start;
            self.available.start += 1;
            Ok(id)
        } else {
            Err(IdAllocationError::OutOfID)
        }
    }

    fn next_id(&mut self) -> Result<[u8; 36], IdAllocationError> {
        let mut buf = [0u8; 36];
        (&mut buf[0..32]).copy_from_slice(&self.transaction_hash.0);
        (&mut buf[32..]).copy_from_slice(&self.next()?.to_le_bytes());
        Ok(buf)
    }

    /// Creates a new package ID.
    pub fn new_package_address(&mut self) -> Result<PackageAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        // println!("Genesis package {:?}", hash(&data).lower_26_bytes());

        Ok(PackageAddress::Normal(hash(data).lower_26_bytes()))
    }

    pub fn new_account_address(&mut self) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        // println!("Genesis account {:?}", hash(&data).lower_26_bytes());

        Ok(ComponentAddress::Account(hash(data).lower_26_bytes()))
    }

    /// Creates a new component address.
    pub fn new_component_address(&mut self) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        // println!("Genesis component {:?}", hash(&data).lower_26_bytes());

        Ok(ComponentAddress::Normal(hash(data).lower_26_bytes()))
    }

    pub fn new_epoch_manager_address(&mut self) -> Result<SystemAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        // println!("Genesis epoch manager {:?}", hash(&data).lower_26_bytes());

        Ok(SystemAddress::EpochManager(hash(data).lower_26_bytes()))
    }

    pub fn new_clock_address(&mut self) -> Result<SystemAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        // println!("Genesis clock {:?}", hash(&data).lower_26_bytes());

        Ok(SystemAddress::Clock(hash(data).lower_26_bytes()))
    }

    /// Creates a new resource address.
    pub fn new_resource_address(&mut self) -> Result<ResourceAddress, IdAllocationError> {
        let mut data = self.transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        // println!("Genesis resource {:?}", hash(&data).lower_26_bytes());

        Ok(ResourceAddress::Normal(hash(data).lower_26_bytes()))
    }

    pub fn new_auth_zone_id(&mut self) -> Result<AuthZoneStackId, IdAllocationError> {
        Ok(self.next()?)
    }

    pub fn new_fee_reserve_id(&mut self) -> Result<FeeReserveId, IdAllocationError> {
        Ok(self.next()?)
    }

    /// Creates a new bucket ID.
    pub fn new_bucket_id(&mut self) -> Result<BucketId, IdAllocationError> {
        Ok(self.next()?)
    }

    /// Creates a new proof ID.
    pub fn new_proof_id(&mut self) -> Result<ProofId, IdAllocationError> {
        Ok(self.next()?)
    }

    /// Creates a new vault ID.
    pub fn new_vault_id(&mut self) -> Result<VaultId, IdAllocationError> {
        self.next_id()
    }

    pub fn new_transaction_hash_id(&mut self) -> Result<TransactionRuntimeId, IdAllocationError> {
        self.next()
    }

    pub fn new_component_id(&mut self) -> Result<ComponentId, IdAllocationError> {
        self.next_id()
    }

    /// Creates a new key value store ID.
    pub fn new_kv_store_id(&mut self) -> Result<KeyValueStoreId, IdAllocationError> {
        self.next_id()
    }

    /// Creates a new non-fungible store ID.
    pub fn new_nf_store_id(&mut self) -> Result<NonFungibleStoreId, IdAllocationError> {
        self.next_id()
    }

    pub fn new_resource_manager_id(&mut self) -> Result<ResourceManagerId, IdAllocationError> {
        self.next_id()
    }

    pub fn new_package_id(&mut self) -> Result<PackageId, IdAllocationError> {
        self.next_id()
    }
}
