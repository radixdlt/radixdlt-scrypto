use crate::data::transform;
use crate::data::TransformHandler;
use crate::errors::*;
use crate::validation::*;
use radix_engine_common::prelude::Reference;
use radix_engine_interface::data::manifest::model::*;
use radix_engine_interface::data::manifest::*;
use radix_engine_interface::data::scrypto::model::Own;
use radix_engine_interface::types::NodeId;
use radix_engine_interface::*;
use sbor::rust::collections::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofKind {
    /// Proof of virtual bucket.
    VirtualProof,
    /// Bucket proof.
    BucketProof(ManifestBucket),
    /// Proof taken or derived from auth zone.
    AuthZoneProof,
}

#[derive(Default)]
pub struct ManifestValidator {
    id_allocator: ManifestIdAllocator,
    /// Bucket id -> lock count
    bucket_ids: NonIterMap<ManifestBucket, usize>,
    /// Proof id to proof info
    proof_ids: NonIterMap<ManifestProof, ProofKind>,
    /// Set of active allocated global address reservation ids
    reservation_ids: IndexSet<ManifestReservation>,
    /// Set of allocated global address ids
    allocated_address_ids: IndexSet<ManifestAllocatedAddress>,
}

impl ManifestValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_bucket(&mut self) -> ManifestBucket {
        let bucket_id = self.id_allocator.new_bucket_id();
        self.bucket_ids.insert(bucket_id.clone(), 0);
        bucket_id
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

        let proof_id = self.id_allocator.new_proof_id();
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
            let proof_id = self.id_allocator.new_proof_id();
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

    pub fn new_reservation(&mut self) -> ManifestReservation {
        let reservation_id = self.id_allocator.new_reservation_id();
        self.reservation_ids.insert(reservation_id.clone());
        reservation_id
    }

    pub fn drop_reservation(
        &mut self,
        reservation_id: &ManifestReservation,
    ) -> Result<(), ManifestIdValidationError> {
        if self.reservation_ids.remove(reservation_id) {
            Ok(())
        } else {
            Err(ManifestIdValidationError::ReservationNotFound(
                reservation_id.clone(),
            ))
        }
    }

    pub fn new_allocated_address(&mut self) -> ManifestAllocatedAddress {
        let allocated_address_id = self.id_allocator.new_allocated_address_id();
        self.allocated_address_ids
            .insert(allocated_address_id.clone());
        allocated_address_id
    }

    pub fn copy_allocated_address(
        &mut self,
        allocated_address_id: &ManifestAllocatedAddress,
    ) -> Result<(), ManifestIdValidationError> {
        if self.allocated_address_ids.contains(allocated_address_id) {
            Ok(())
        } else {
            Err(ManifestIdValidationError::AllocatedAddressNotFound(
                allocated_address_id.clone(),
            ))
        }
    }

    pub fn process_call_data(
        &mut self,
        args: &ManifestValue,
    ) -> Result<(), ManifestIdValidationError> {
        transform(args.clone(), self).map(|_| ())
    }
}

impl TransformHandler<ManifestIdValidationError> for ManifestValidator {
    fn replace_bucket(&mut self, b: ManifestBucket) -> Result<Own, ManifestIdValidationError> {
        self.drop_bucket(&b)?;
        Ok(Own(NodeId([0u8; NodeId::LENGTH])))
    }

    fn replace_proof(&mut self, p: ManifestProof) -> Result<Own, ManifestIdValidationError> {
        self.drop_proof(&p)?;
        Ok(Own(NodeId([0u8; NodeId::LENGTH])))
    }

    fn replace_reservation(
        &mut self,
        r: ManifestReservation,
    ) -> Result<Own, ManifestIdValidationError> {
        self.drop_reservation(&r)?;
        Ok(Own(NodeId([0u8; NodeId::LENGTH])))
    }

    fn replace_allocated_address(
        &mut self,
        a: ManifestAllocatedAddress,
    ) -> Result<Reference, ManifestIdValidationError> {
        self.copy_allocated_address(&a)?;
        Ok(Reference(NodeId([0u8; NodeId::LENGTH])))
    }

    // TODO: validate expression and blob as well

    fn replace_expression(
        &mut self,
        _e: ManifestExpression,
    ) -> Result<Vec<Own>, ManifestIdValidationError> {
        Ok(Vec::new())
    }

    fn replace_blob(&mut self, _b: ManifestBlobRef) -> Result<Vec<u8>, ManifestIdValidationError> {
        Ok(Vec::new())
    }
}
