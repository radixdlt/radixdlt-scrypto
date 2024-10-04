use crate::data::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofKind {
    /// Proof created from a bucket
    BucketProof(ManifestBucket),
    /// Proof created from auth zone.
    AuthZoneProof,
}

/// For advanced validation, you should consider using the
/// [`StaticManifestInterpreter`] instead.
#[derive(Default)]
pub struct BasicManifestValidator {
    id_allocator: ManifestIdAllocator,
    /// Bucket id -> lock count
    bucket_ids: NonIterMap<ManifestBucket, usize>,
    /// Proof id to proof info
    proof_ids: BTreeMap<ManifestProof, ProofKind>,
    /// Set of active allocated global address reservation ids
    address_reservation_ids: IndexSet<ManifestAddressReservation>,
    /// Set of named global address ids
    address_ids: IndexSet<ManifestNamedAddress>,
    /// Set of named intent hashes
    intent_ids: IndexSet<ManifestNamedIntent>,
}

impl BasicManifestValidator {
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
            ProofKind::AuthZoneProof => {}
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

    pub fn drop_all_named_proofs(&mut self) -> Result<(), ManifestIdValidationError> {
        let proof_ids = self.proof_ids.keys().copied().collect::<Vec<_>>();
        for proof_id in proof_ids {
            self.drop_proof(&proof_id)?
        }
        Ok(())
    }

    pub fn new_address_reservation(&mut self) -> ManifestAddressReservation {
        let address_reservation_id = self.id_allocator.new_address_reservation_id();
        self.address_reservation_ids
            .insert(address_reservation_id.clone());
        address_reservation_id
    }

    pub fn drop_address_reservation(
        &mut self,
        address_reservation_id: &ManifestAddressReservation,
    ) -> Result<(), ManifestIdValidationError> {
        if self
            .address_reservation_ids
            .swap_remove(address_reservation_id)
        {
            Ok(())
        } else {
            Err(ManifestIdValidationError::AddressReservationNotFound(
                address_reservation_id.clone(),
            ))
        }
    }

    pub fn new_named_address(&mut self) -> ManifestNamedAddress {
        let address_id = self.id_allocator.new_address_id();
        self.address_ids.insert(address_id.clone());
        address_id
    }

    pub fn check_bucket(
        &mut self,
        bucket_id: &ManifestBucket,
    ) -> Result<(), ManifestIdValidationError> {
        if self.bucket_ids.contains_key(bucket_id) {
            Ok(())
        } else {
            Err(ManifestIdValidationError::BucketNotFound(bucket_id.clone()))
        }
    }

    pub fn check_named_address(
        &mut self,
        address_id: &ManifestNamedAddress,
    ) -> Result<(), ManifestIdValidationError> {
        if self.address_ids.contains(address_id) {
            Ok(())
        } else {
            Err(ManifestIdValidationError::AddressNotFound(
                address_id.clone(),
            ))
        }
    }

    pub fn new_intent(&mut self) -> ManifestNamedIntent {
        let intent_id = self.id_allocator.new_named_intent_id();
        self.intent_ids.insert(intent_id.clone());
        intent_id
    }

    pub fn process_call_data(
        &mut self,
        args: &ManifestValue,
    ) -> Result<(), ManifestIdValidationError> {
        transform(args.clone(), self).map(|_| ())
    }
}

impl TransformHandler<ManifestIdValidationError> for BasicManifestValidator {
    fn replace_bucket(&mut self, b: ManifestBucket) -> Result<Own, ManifestIdValidationError> {
        self.drop_bucket(&b)?;
        Ok(Own(NodeId([0u8; NodeId::LENGTH])))
    }

    fn replace_proof(&mut self, p: ManifestProof) -> Result<Own, ManifestIdValidationError> {
        self.drop_proof(&p)?;
        Ok(Own(NodeId([0u8; NodeId::LENGTH])))
    }

    fn replace_address_reservation(
        &mut self,
        r: ManifestAddressReservation,
    ) -> Result<Own, ManifestIdValidationError> {
        self.drop_address_reservation(&r)?;
        Ok(Own(NodeId([0u8; NodeId::LENGTH])))
    }

    fn replace_named_address(
        &mut self,
        a: ManifestNamedAddress,
    ) -> Result<Reference, ManifestIdValidationError> {
        self.check_named_address(&a)?;
        Ok(Reference(NodeId([0u8; NodeId::LENGTH])))
    }

    fn replace_expression(
        &mut self,
        _e: ManifestExpression,
    ) -> Result<Vec<Own>, ManifestIdValidationError> {
        Ok(Vec::new())
    }

    // TODO: validate blob as well?
    fn replace_blob(&mut self, _b: ManifestBlobRef) -> Result<Vec<u8>, ManifestIdValidationError> {
        Ok(Vec::new())
    }
}
