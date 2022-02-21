use scrypto::engine::types::*;
use scrypto::rust::collections::*;

use crate::engine::*;
use crate::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdValidatorError {
    IdAllocatorError(IdAllocatorError),
    BucketNotFound(BucketId),
    ProofNotFound(ProofId),
    BucketLocked(BucketId),
}

pub struct IdValidator {
    id_allocator: IdAllocator,
    bucket_ids: HashMap<BucketId, usize>,
    proof_ids: HashMap<ProofId, BucketId>,
}

impl IdValidator {
    pub fn new() -> Self {
        let mut proof_ids = HashMap::new();
        proof_ids.insert(ECDSA_TOKEN_PROOF_ID, ECDSA_TOKEN_BUCKET_ID);
        Self {
            id_allocator: IdAllocator::new(IdSpace::Transaction),
            bucket_ids: HashMap::new(),
            proof_ids,
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

    pub fn new_proof(&mut self, bucket_id: BucketId) -> Result<ProofId, IdValidatorError> {
        if let Some(cnt) = self.bucket_ids.get_mut(&bucket_id) {
            *cnt += 1;
            let proof_id = self
                .id_allocator
                .new_proof_id()
                .map_err(IdValidatorError::IdAllocatorError)?;
            self.proof_ids.insert(proof_id, bucket_id);
            Ok(proof_id)
        } else {
            Err(IdValidatorError::BucketNotFound(bucket_id))
        }
    }

    pub fn clone_proof(&mut self, proof_id: ProofId) -> Result<ProofId, IdValidatorError> {
        if let Some(bucket_id) = self.proof_ids.get(&proof_id).cloned() {
            // for virtual badge, the corresponding bucket is not owned by transaction.
            if let Some(cnt) = self.bucket_ids.get_mut(&bucket_id) {
                *cnt += 1;
            }
            let proof_id = self
                .id_allocator
                .new_proof_id()
                .map_err(IdValidatorError::IdAllocatorError)?;
            self.proof_ids.insert(proof_id, bucket_id);
            Ok(proof_id)
        } else {
            Err(IdValidatorError::ProofNotFound(proof_id))
        }
    }

    pub fn drop_proof(&mut self, proof_id: ProofId) -> Result<(), IdValidatorError> {
        if let Some(bucket_id) = self.proof_ids.remove(&proof_id) {
            // for virtual badge, the corresponding bucket is not owned by transaction.
            if let Some(cnt) = self.bucket_ids.get_mut(&bucket_id) {
                *cnt -= 1;
            }
            Ok(())
        } else {
            Err(IdValidatorError::ProofNotFound(proof_id))
        }
    }

    pub fn move_all_resources(&mut self) -> Result<(), IdValidatorError> {
        self.proof_ids.clear();
        self.bucket_ids.clear();
        Ok(())
    }

    pub fn move_resources(&mut self, arg: &ValidatedData) -> Result<(), IdValidatorError> {
        for bucket_id in &arg.bucket_ids {
            self.drop_bucket(*bucket_id)?;
        }
        for proof_id in &arg.proof_ids {
            self.drop_proof(*proof_id)?;
        }
        Ok(())
    }
}
