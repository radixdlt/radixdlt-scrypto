use sbor::rust::collections::*;
use scrypto::engine::types::*;
use scrypto::values::*;

use crate::errors::*;
use crate::validation::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofKind {
    /// Proof of virtual bucket.
    VirtualProof,
    /// Bucket proof.
    BucketProof(BucketId),
    /// Proof taken or derived from auth zone.
    AuthZoneProof,
}

pub struct IdValidator {
    id_allocator: IdAllocator,
    bucket_ids: HashMap<BucketId, usize>,
    proof_ids: HashMap<ProofId, ProofKind>,
}

impl IdValidator {
    pub fn new() -> Self {
        Self {
            id_allocator: IdAllocator::new(IdSpace::Transaction),
            bucket_ids: HashMap::new(),
            proof_ids: HashMap::new(),
        }
    }

    pub fn new_bucket(&mut self) -> Result<BucketId, IdValidationError> {
        let bucket_id = self
            .id_allocator
            .new_bucket_id()
            .map_err(IdValidationError::IdAllocationError)?;
        self.bucket_ids.insert(bucket_id, 0);
        Ok(bucket_id)
    }

    pub fn drop_bucket(&mut self, bucket_id: BucketId) -> Result<(), IdValidationError> {
        if let Some(cnt) = self.bucket_ids.get(&bucket_id) {
            if *cnt == 0 {
                self.bucket_ids.remove(&bucket_id);
                Ok(())
            } else {
                Err(IdValidationError::BucketLocked(bucket_id))
            }
        } else {
            Err(IdValidationError::BucketNotFound(bucket_id))
        }
    }

    pub fn new_proof(&mut self, kind: ProofKind) -> Result<ProofId, IdValidationError> {
        match &kind {
            ProofKind::BucketProof(bucket_id) => {
                if let Some(cnt) = self.bucket_ids.get_mut(bucket_id) {
                    *cnt += 1;
                } else {
                    return Err(IdValidationError::BucketNotFound(*bucket_id));
                }
            }
            ProofKind::AuthZoneProof | ProofKind::VirtualProof => {}
        }

        let proof_id = self
            .id_allocator
            .new_proof_id()
            .map_err(IdValidationError::IdAllocationError)?;
        self.proof_ids.insert(proof_id, kind);
        Ok(proof_id)
    }

    pub fn clone_proof(&mut self, proof_id: ProofId) -> Result<ProofId, IdValidationError> {
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
                .map_err(IdValidationError::IdAllocationError)?;
            self.proof_ids.insert(proof_id, kind);
            Ok(proof_id)
        } else {
            Err(IdValidationError::ProofNotFound(proof_id))
        }
    }

    pub fn drop_proof(&mut self, proof_id: ProofId) -> Result<(), IdValidationError> {
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
            Err(IdValidationError::ProofNotFound(proof_id))
        }
    }

    pub fn drop_all_proofs(&mut self) -> Result<(), IdValidationError> {
        self.proof_ids.clear();
        Ok(())
    }

    pub fn move_all_buckets(&mut self) -> Result<(), IdValidationError> {
        self.bucket_ids.clear();
        Ok(())
    }

    pub fn move_resources(&mut self, arg: &ScryptoValue) -> Result<(), IdValidationError> {
        for (bucket_id, _) in &arg.bucket_ids {
            self.drop_bucket(*bucket_id)?;
        }
        for (proof_id, _) in &arg.proof_ids {
            self.drop_proof(*proof_id)?;
        }
        Ok(())
    }
}
