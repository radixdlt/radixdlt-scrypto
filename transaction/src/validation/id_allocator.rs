use radix_engine_lib::model::*;
use radix_engine_lib::component::PackageAddress;
use radix_engine_lib::component::SystemAddress;
use radix_engine_lib::crypto::{hash, Hash};
use radix_engine_lib::engine::types::{
    AuthZoneId, BucketId, ComponentId, KeyValueStoreId, NonFungibleStoreId, PackageId, ProofId,
    ResourceManagerId, VaultId,
};
use radix_engine_lib::resource::ResourceAddress;
use sbor::rust::ops::Range;
use scrypto::constants::*;

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
}

impl IdAllocator {
    /// Creates an ID allocator.
    pub fn new(kind: IdSpace) -> Self {
        Self {
            available: match kind {
                IdSpace::System => 0..512,
                IdSpace::Transaction => 512..1024,
                IdSpace::Application => 1024..u32::MAX,
            },
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

    fn next_id(&mut self, transaction_hash: Hash) -> Result<[u8; 36], IdAllocationError> {
        let mut buf = [0u8; 36];
        (&mut buf[0..32]).copy_from_slice(&transaction_hash.0);
        (&mut buf[32..]).copy_from_slice(&self.next()?.to_le_bytes());
        Ok(buf)
    }

    /// Creates a new package ID.
    pub fn new_package_address(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<PackageAddress, IdAllocationError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        // println!("Genesis package {:?}", hash(&data).lower_26_bytes());

        Ok(PackageAddress::Normal(hash(data).lower_26_bytes()))
    }

    /// Creates a new component address.
    pub fn new_component_address(
        &mut self,
        transaction_hash: Hash,
        package_address: PackageAddress,
        blueprint_name: &str,
    ) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        // println!("Genesis component {:?}", hash(&data).lower_26_bytes());

        match (package_address, blueprint_name) {
            (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => {
                Ok(ComponentAddress::Account(hash(data).lower_26_bytes()))
            }
            _ => Ok(ComponentAddress::Normal(hash(data).lower_26_bytes())),
        }
    }

    pub fn new_system_address(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<SystemAddress, IdAllocationError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        // println!("Genesis system {:?}", hash(&data).lower_26_bytes());

        Ok(SystemAddress::EpochManager(hash(data).lower_26_bytes()))
    }

    /// Creates a new resource address.
    pub fn new_resource_address(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<ResourceAddress, IdAllocationError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        // println!("Genesis resource {:?}", hash(&data).lower_26_bytes());

        Ok(ResourceAddress::Normal(hash(data).lower_26_bytes()))
    }

    /// Creates a new UUID.
    pub fn new_uuid(&mut self, transaction_hash: Hash) -> Result<u128, IdAllocationError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(u128::from_le_bytes(hash(data).lower_16_bytes()))
    }

    pub fn new_auth_zone_id(&mut self) -> Result<AuthZoneId, IdAllocationError> {
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
    pub fn new_vault_id(&mut self, transaction_hash: Hash) -> Result<VaultId, IdAllocationError> {
        self.next_id(transaction_hash)
    }

    pub fn new_component_id(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<ComponentId, IdAllocationError> {
        self.next_id(transaction_hash)
    }

    /// Creates a new key value store ID.
    pub fn new_kv_store_id(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<KeyValueStoreId, IdAllocationError> {
        self.next_id(transaction_hash)
    }

    /// Creates a new non-fungible store ID.
    pub fn new_nf_store_id(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<NonFungibleStoreId, IdAllocationError> {
        self.next_id(transaction_hash)
    }

    pub fn new_resource_manager_id(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<ResourceManagerId, IdAllocationError> {
        self.next_id(transaction_hash)
    }

    pub fn new_package_id(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<PackageId, IdAllocationError> {
        self.next_id(transaction_hash)
    }
}
