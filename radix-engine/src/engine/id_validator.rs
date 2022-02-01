use scrypto::rust::collections::*;
use scrypto::types::*;

use crate::engine::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdValidatorError {
    IdAllocatorError(IdAllocatorError),
    BucketNotFound(Bid),
    BucketRefNotFound(Rid),
    BucketLocked(Bid),
}

pub struct IdValidator {
    id_allocator: IdAllocator,
    buckets: HashMap<Bid, usize>,
    bucket_refs: HashMap<Rid, Bid>,
}

impl IdValidator {
    pub fn new() -> Self {
        let mut bucket_refs = HashMap::new();
        bucket_refs.insert(ECDSA_TOKEN_RID, ECDSA_TOKEN_BID);
        Self {
            id_allocator: IdAllocator::new(TRANSACTION_ID_SPACE),
            buckets: HashMap::new(),
            bucket_refs,
        }
    }

    pub fn new_bucket(&mut self) -> Result<Bid, IdValidatorError> {
        let bid = self
            .id_allocator
            .new_bid()
            .map_err(IdValidatorError::IdAllocatorError)?;
        self.buckets.insert(bid, 0);
        Ok(bid)
    }

    pub fn drop_bucket(&mut self, bid: Bid) -> Result<(), IdValidatorError> {
        if let Some(cnt) = self.buckets.get(&bid) {
            if *cnt == 0 {
                self.buckets.remove(&bid);
                Ok(())
            } else {
                Err(IdValidatorError::BucketLocked(bid))
            }
        } else {
            Err(IdValidatorError::BucketNotFound(bid))
        }
    }

    pub fn new_bucket_ref(&mut self, bid: Bid) -> Result<Rid, IdValidatorError> {
        if let Some(cnt) = self.buckets.get_mut(&bid) {
            *cnt += 1;
            let rid = self
                .id_allocator
                .new_rid()
                .map_err(IdValidatorError::IdAllocatorError)?;
            self.bucket_refs.insert(rid, bid);
            Ok(rid)
        } else {
            Err(IdValidatorError::BucketNotFound(bid))
        }
    }

    pub fn clone_bucket_ref(&mut self, rid: Rid) -> Result<Rid, IdValidatorError> {
        if let Some(bid) = self.bucket_refs.get(&rid).cloned() {
            // for virtual badge, the corresponding bucket is not owned by transaction.
            if let Some(cnt) = self.buckets.get_mut(&bid) {
                *cnt += 1;
            }
            let rid = self
                .id_allocator
                .new_rid()
                .map_err(IdValidatorError::IdAllocatorError)?;
            self.bucket_refs.insert(rid, bid);
            Ok(rid)
        } else {
            Err(IdValidatorError::BucketRefNotFound(rid))
        }
    }

    pub fn drop_bucket_ref(&mut self, rid: Rid) -> Result<(), IdValidatorError> {
        if let Some(bid) = self.bucket_refs.remove(&rid) {
            // for virtual badge, the corresponding bucket is not owned by transaction.
            if let Some(cnt) = self.buckets.get_mut(&bid) {
                *cnt -= 1;
            }
            Ok(())
        } else {
            Err(IdValidatorError::BucketRefNotFound(rid))
        }
    }

    pub fn drop_all(&mut self) -> Result<(), IdValidatorError> {
        self.bucket_refs.clear();
        self.buckets.clear();
        Ok(())
    }
}
