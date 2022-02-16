use scrypto::crypto::sha256_twice;
use scrypto::engine::types::*;
use scrypto::rust::ops::Range;

pub const ECDSA_TOKEN_BUCKET_ID: BucketId = 0;
pub const ECDSA_TOKEN_PROOF_ID: ProofId = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdSpace {
    System,
    Transaction,
    Application,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdAllocatorError {
    OutOfID,
}

/// An ID allocator defines how identities are generated.
pub struct IdAllocator {
    available: Range<u32>,
}

impl IdAllocator {
    /// Creates an ID allocator.
    pub fn new(kind: IdSpace) -> Self {
        Self {
            available: match kind {
                IdSpace::System => (0..512),
                IdSpace::Transaction => (512..1024),
                IdSpace::Application => (1024..u32::MAX),
            },
        }
    }

    fn next(&mut self) -> Result<u32, IdAllocatorError> {
        if self.available.len() > 0 {
            let id = self.available.start;
            self.available.start += 1;
            Ok(id)
        } else {
            Err(IdAllocatorError::OutOfID)
        }
    }

    /// Creates a new package ref.
    pub fn new_package_ref(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<PackageRef, IdAllocatorError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(PackageRef(sha256_twice(data).lower_26_bytes()))
    }

    /// Creates a new component ref.
    pub fn new_component_ref(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<ComponentRef, IdAllocatorError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(ComponentRef(sha256_twice(data).lower_26_bytes()))
    }

    /// Creates a new resource def ref.
    pub fn new_resource_def_ref(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<ResourceDefRef, IdAllocatorError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(ResourceDefRef(sha256_twice(data).lower_26_bytes()))
    }

    /// Creates a new UUID.
    pub fn new_uuid(&mut self, transaction_hash: Hash) -> Result<u128, IdAllocatorError> {
        let mut data = transaction_hash.to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(u128::from_le_bytes(sha256_twice(data).lower_16_bytes()))
    }

    /// Creates a new bucket ID.
    pub fn new_bucket_id(&mut self) -> Result<BucketId, IdAllocatorError> {
        Ok(self.next()?)
    }

    /// Creates a new proof ID.
    pub fn new_proof_id(&mut self) -> Result<ProofId, IdAllocatorError> {
        Ok(self.next()?)
    }

    /// Creates a new vault ID.
    pub fn new_vault_id(&mut self, transaction_hash: Hash) -> Result<VaultId, IdAllocatorError> {
        Ok((transaction_hash, self.next()?))
    }

    /// Creates a new lazy map ID.
    pub fn new_lazy_map_id(
        &mut self,
        transaction_hash: Hash,
    ) -> Result<LazyMapId, IdAllocatorError> {
        Ok((transaction_hash, self.next()?))
    }
}
