use scrypto::engine::types::*;
use scrypto::rust::collections::*;

use crate::engine::*;
use crate::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdValidatorError {
    IdAllocatorError(IdAllocatorError),
    BucketNotFound(BucketId),
    BucketRefNotFound(BucketRefId),
    BucketLocked(BucketId),
}

pub struct IdValidator {
    id_allocator: IdAllocator,
    bucket_ids: HashMap<BucketId, usize>,
    bucket_ref_ids: HashMap<BucketRefId, BucketId>,
}

impl IdValidator {
    pub fn new() -> Self {
        let mut bucket_ref_ids = HashMap::new();
        bucket_ref_ids.insert(ECDSA_TOKEN_BUCKET_REF_ID, ECDSA_TOKEN_BUCKET_ID);
        Self {
            id_allocator: IdAllocator::new(IdSpace::Transaction),
            bucket_ids: HashMap::new(),
            bucket_ref_ids,
        }
    }

    pub fn new_bucket(&mut self) -> Result<BucketId, IdValidatorError> {
        let bucket_id = self
            .id_allocator
            .new_bucket_id()
            .map_err(IdValidatorError::IdAllocatorError)?;
        self.bucket_ids.insert(bucket_id, 0);
        Ok(bucket_id)
    }

    pub fn drop_bucket(&mut self, bucket_id: BucketId) -> Result<(), IdValidatorError> {
        if let Some(cnt) = self.bucket_ids.get(&bucket_id) {
            if *cnt == 0 {
                self.bucket_ids.remove(&bucket_id);
                Ok(())
            } else {
                Err(IdValidatorError::BucketLocked(bucket_id))
            }
        } else {
            Err(IdValidatorError::BucketNotFound(bucket_id))
        }
    }

    pub fn new_bucket_ref(&mut self, bucket_id: BucketId) -> Result<BucketRefId, IdValidatorError> {
        if let Some(cnt) = self.bucket_ids.get_mut(&bucket_id) {
            *cnt += 1;
            let bucket_ref_id = self
                .id_allocator
                .new_bucket_ref_id()
                .map_err(IdValidatorError::IdAllocatorError)?;
            self.bucket_ref_ids.insert(bucket_ref_id, bucket_id);
            Ok(bucket_ref_id)
        } else {
            Err(IdValidatorError::BucketNotFound(bucket_id))
        }
    }

    pub fn clone_bucket_ref(
        &mut self,
        bucket_ref_id: BucketRefId,
    ) -> Result<BucketRefId, IdValidatorError> {
        if let Some(bucket_id) = self.bucket_ref_ids.get(&bucket_ref_id).cloned() {
            // for virtual badge, the corresponding bucket is not owned by transaction.
            if let Some(cnt) = self.bucket_ids.get_mut(&bucket_id) {
                *cnt += 1;
            }
            let bucket_ref_id = self
                .id_allocator
                .new_bucket_ref_id()
                .map_err(IdValidatorError::IdAllocatorError)?;
            self.bucket_ref_ids.insert(bucket_ref_id, bucket_id);
            Ok(bucket_ref_id)
        } else {
            Err(IdValidatorError::BucketRefNotFound(bucket_ref_id))
        }
    }

    pub fn drop_bucket_ref(&mut self, bucket_ref_id: BucketRefId) -> Result<(), IdValidatorError> {
        if let Some(bucket_id) = self.bucket_ref_ids.remove(&bucket_ref_id) {
            // for virtual badge, the corresponding bucket is not owned by transaction.
            if let Some(cnt) = self.bucket_ids.get_mut(&bucket_id) {
                *cnt -= 1;
            }
            Ok(())
        } else {
            Err(IdValidatorError::BucketRefNotFound(bucket_ref_id))
        }
    }

    pub fn move_all_resources(&mut self) -> Result<(), IdValidatorError> {
        self.bucket_ref_ids.clear();
        self.bucket_ids.clear();
        Ok(())
    }

    pub fn move_resources(&mut self, arg: &ValidatedData) -> Result<(), IdValidatorError> {
        for bucket_id in &arg.bucket_ids {
            self.drop_bucket(*bucket_id)?;
        }
        for bucket_ref_id in &arg.bucket_ref_ids {
            self.drop_bucket_ref(*bucket_ref_id)?;
        }
        Ok(())
    }
}
