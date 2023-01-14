use radix_engine_interface::data::types::ManifestBucket;
use radix_engine_interface::data::types::ManifestProof;
use sbor::rust::collections::*;
use sbor::SborPath;

use crate::errors::*;
use crate::validation::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofKind {
    /// Proof of virtual bucket.
    VirtualProof,
    /// Bucket proof.
    BucketProof(ManifestBucket),
    /// Proof taken or derived from auth zone.
    AuthZoneProof,
}

pub struct ManifestIdValidator {
    id_allocator: ManifestIdAllocator,
    bucket_ids: HashMap<ManifestBucket, usize>,
    proof_ids: HashMap<ManifestProof, ProofKind>,
}

impl ManifestIdValidator {
    pub fn new() -> Self {
        Self {
            id_allocator: ManifestIdAllocator::new(),
            bucket_ids: HashMap::new(),
            proof_ids: HashMap::new(),
        }
    }

    pub fn new_bucket(&mut self) -> Result<ManifestBucket, ManifestIdValidationError> {
        let bucket_id = self
            .id_allocator
            .new_bucket_id()
            .map_err(ManifestIdValidationError::IdAllocationError)?;
        self.bucket_ids.insert(bucket_id.clone(), 0);
        Ok(bucket_id)
    }

    pub fn drop_bucket(
        &mut self,
        bucket_id: &ManifestBucket,
    ) -> Result<(), ManifestIdValidationError> {
        if let Some(cnt) = self.bucket_ids.get(bucket_id) {
            if *cnt == 0 {
                self.bucket_ids.remove(bucket_id);
                Ok(())
            } else {
                Err(ManifestIdValidationError::BucketLocked(bucket_id.clone()))
            }
        } else {
            Err(ManifestIdValidationError::BucketNotFound(bucket_id.clone()))
        }
    }

    pub fn new_proof(
        &mut self,
        kind: ProofKind,
    ) -> Result<ManifestProof, ManifestIdValidationError> {
        match &kind {
            ProofKind::BucketProof(bucket_id) => {
                if let Some(cnt) = self.bucket_ids.get_mut(bucket_id) {
                    *cnt += 1;
                } else {
                    return Err(ManifestIdValidationError::BucketNotFound(bucket_id.clone()));
                }
            }
            ProofKind::AuthZoneProof | ProofKind::VirtualProof => {}
        }

        let proof_id = self
            .id_allocator
            .new_proof_id()
            .map_err(ManifestIdValidationError::IdAllocationError)?;
        self.proof_ids.insert(proof_id.clone(), kind);
        Ok(proof_id)
    }

    pub fn clone_proof(
        &mut self,
        proof_id: &ManifestProof,
    ) -> Result<ManifestProof, ManifestIdValidationError> {
        if let Some(kind) = self.proof_ids.get(proof_id).cloned() {
            if let ProofKind::BucketProof(bucket_id) = &kind {
                if let Some(cnt) = self.bucket_ids.get_mut(bucket_id) {
                    *cnt += 1;
                } else {
                    panic!("Illegal state");
                }
            }
            let proof_id = self
                .id_allocator
                .new_proof_id()
                .map_err(ManifestIdValidationError::IdAllocationError)?;
            self.proof_ids.insert(proof_id.clone(), kind);
            Ok(proof_id)
        } else {
            Err(ManifestIdValidationError::ProofNotFound(proof_id.clone()))
        }
    }

    pub fn drop_proof(
        &mut self,
        proof_id: &ManifestProof,
    ) -> Result<(), ManifestIdValidationError> {
        if let Some(kind) = self.proof_ids.remove(proof_id) {
            if let ProofKind::BucketProof(bucket_id) = kind {
                if let Some(cnt) = self.bucket_ids.get_mut(&bucket_id) {
                    *cnt -= 1;
                } else {
                    panic!("Illegal state");
                }
            }
            Ok(())
        } else {
            Err(ManifestIdValidationError::ProofNotFound(proof_id.clone()))
        }
    }

    pub fn drop_all_proofs(&mut self) -> Result<(), ManifestIdValidationError> {
        self.proof_ids.clear();
        Ok(())
    }

    pub fn move_resources(
        &mut self,
        buckets: &Vec<(ManifestBucket, SborPath)>,
        proofs: &Vec<(ManifestProof, SborPath)>,
    ) -> Result<(), ManifestIdValidationError> {
        for (bucket, _) in buckets {
            self.drop_bucket(bucket)?;
        }
        for (proof, _) in proofs {
            self.drop_proof(proof)?;
        }
        Ok(())
    }
}
