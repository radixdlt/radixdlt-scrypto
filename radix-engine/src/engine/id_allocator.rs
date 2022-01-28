use scrypto::types::*;
use scrypto::utils::*;

pub const ECDSA_TOKEN_BID: Bid = Bid(0);
pub const ECDSA_TOKEN_RID: Rid = Rid(1);

pub const SYSTEM_OBJECT_ID_RANGE: (u32, u32) = (0, 512);
pub const TRANSACTION_OBJECT_ID_RANGE: (u32, u32) = (512, 1024);
pub const USER_OBJECT_ID_RANGE: (u32, u32) = (1024, u32::MAX);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdAllocatorError {
    OutOfID,
}

/// An ID allocator defines how identities are generated.
pub struct IdAllocator {
    next: u32,
    end: u32,
}

impl IdAllocator {
    /// Creates an ID allocator.
    pub fn new(range: (u32, u32)) -> Self {
        assert!(range.1 > range.0);
        Self {
            next: range.0,
            end: range.1,
        }
    }

    fn next(&mut self) -> Result<u32, IdAllocatorError> {
        if self.next < self.end {
            let id = self.next;
            self.next += 1;
            Ok(id)
        } else {
            Err(IdAllocatorError::OutOfID)
        }
    }

    /// Creates a new package address.
    pub fn new_package_address(
        &mut self,
        transaction_hash: H256,
    ) -> Result<Address, IdAllocatorError> {
        let mut data = transaction_hash.as_ref().to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(Address::Package(sha256_twice(data).lower_26_bytes()))
    }

    /// Creates a new component address.
    pub fn new_component_address(
        &mut self,
        transaction_hash: H256,
    ) -> Result<Address, IdAllocatorError> {
        let mut data = transaction_hash.as_ref().to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(Address::Component(sha256_twice(data).lower_26_bytes()))
    }

    /// Creates a new resource def address.
    pub fn new_resource_address(
        &mut self,
        transaction_hash: H256,
    ) -> Result<Address, IdAllocatorError> {
        let mut data = transaction_hash.as_ref().to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(Address::ResourceDef(sha256_twice(data).lower_26_bytes()))
    }

    /// Creates a new UUID.
    pub fn new_uuid(&mut self, transaction_hash: H256) -> Result<u128, IdAllocatorError> {
        let mut data = transaction_hash.as_ref().to_vec();
        data.extend(self.next()?.to_le_bytes());
        Ok(u128::from_le_bytes(sha256_twice(data).lower_16_bytes()))
    }

    /// Creates a new bucket ID.
    pub fn new_bid(&mut self) -> Result<Bid, IdAllocatorError> {
        Ok(Bid(self.next()?))
    }

    /// Creates a new bucket ref ID.
    pub fn new_rid(&mut self) -> Result<Rid, IdAllocatorError> {
        Ok(Rid(self.next()?))
    }

    /// Creates a new vault ID.
    pub fn new_vid(&mut self, transaction_hash: H256) -> Result<Vid, IdAllocatorError> {
        Ok(Vid(transaction_hash, self.next()?))
    }

    /// Creates a new lazy map ID.
    pub fn new_mid(&mut self, transaction_hash: H256) -> Result<Mid, IdAllocatorError> {
        Ok(Mid(transaction_hash, self.next()?))
    }
}
