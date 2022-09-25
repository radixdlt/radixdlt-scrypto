use sbor::rust::ops::Range;
use scrypto::component::{ComponentAddress, PackageAddress};
use scrypto::constants::*;
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::resource::ResourceAddress;

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

    /// Creates a new package ID.
    pub fn new_package_address(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<PackageAddress, IdAllocationError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        Ok(PackageAddress::Normal(hash(data).lower_26_bytes()))
    }

    /// Creates a new component address.
    pub fn new_component_address(
        &mut self,
        transaction_hash: Hash,
        package_address: &PackageAddress,
        blueprint_name: &str,
    ) -> Result<ComponentAddress, IdAllocationError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        match (*package_address, blueprint_name) {
            (ACCOUNT_PACKAGE, "Account") => {
                Ok(ComponentAddress::Account(hash(data).lower_26_bytes()))
            }
            (SYS_FAUCET_PACKAGE, "SysFaucet") => {
                Ok(ComponentAddress::System(hash(data).lower_26_bytes()))
            }
            _ => Ok(ComponentAddress::Normal(hash(data).lower_26_bytes())),
        }
    }

    /// Creates a new resource address.
    pub fn new_resource_address(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<ResourceAddress, IdAllocationError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());

        Ok(ResourceAddress::Normal(hash(data).lower_26_bytes()))
    }

    /// Creates a new UUID.
    pub fn new_uuid(&mut self, transaction_hash: Hash) -> Result<u128, IdAllocationError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(u128::from_le_bytes(hash(data).lower_16_bytes()))
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
        Ok((transaction_hash, self.next()?))
    }

    /// Creates a new key value store ID.
    pub fn new_kv_store_id(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<KeyValueStoreId, IdAllocationError> {
        Ok((transaction_hash, self.next()?))
    }
}
