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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofKind {
    /// Proof of virtual bucket.
    VirtualProof,
    /// Bucket proof.
    BucketProof(BucketId),
    /// Proof taken from auth worktop.
    RuntimeProof,
}

pub struct IdValidator {
    id_allocator: IdAllocator,
    bucket_ids: HashMap<BucketId, usize>,
    proof_ids: HashMap<ProofId, ProofKind>,
}

impl IdValidator {
    pub fn new() -> Self {
        let mut proof_ids = HashMap::new();
        proof_ids.insert(ECDSA_TOKEN_PROOF_ID, ProofKind::VirtualProof);
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

    pub fn new_proof(&mut self, kind: ProofKind) -> Result<ProofId, IdValidatorError> {
        match &kind {
            ProofKind::BucketProof(bucket_id) => {
                if let Some(cnt) = self.bucket_ids.get_mut(bucket_id) {
                    *cnt += 1;
                } else {
                    return Err(IdValidatorError::BucketNotFound(*bucket_id));
                }
            }
            ProofKind::RuntimeProof | ProofKind::VirtualProof => {}
        }

        let proof_id = self
            .id_allocator
            .new_proof_id()
            .map_err(IdValidatorError::IdAllocatorError)?;
        self.proof_ids.insert(proof_id, kind);
        Ok(proof_id)
    }

    pub fn clone_proof(&mut self, proof_id: ProofId) -> Result<ProofId, IdValidatorError> {
        if let Some(kind) = self.proof_ids.get(&proof_id).cloned() {
            if let ProofKind::BucketProof(bucket_id) = kind {
                if let Some(cnt) = self.bucket_ids.get_mut(&bucket_id) {
                    *cnt += 1;
                } else {
                    panic!("Illegal state");
                }
            }
            let proof_id = self
                .id_allocator
                .new_proof_id()
                .map_err(IdValidatorError::IdAllocatorError)?;
            self.proof_ids.insert(proof_id, kind);
            Ok(proof_id)
        } else {
            Err(IdValidatorError::ProofNotFound(proof_id))
        }
    }

    pub fn drop_proof(&mut self, proof_id: ProofId) -> Result<(), IdValidatorError> {
        if let Some(kind) = self.proof_ids.remove(&proof_id) {
            if let ProofKind::BucketProof(bucket_id) = kind {
                if let Some(cnt) = self.bucket_ids.get_mut(&bucket_id) {
                    *cnt -= 1;
                } else {
                    panic!("Illegal state");
                }
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
