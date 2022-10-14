use crate::model::{
    AuthZoneError, InvokeError, LockableResource, LockedAmountOrIds, ProofSubstate,
    ResourceContainerId,
};
use crate::types::*;

/// A transient resource container.
#[derive(Debug)]
pub struct AuthZoneSubstate {
    pub proofs: Vec<ProofSubstate>,
    /// IDs of buckets that act as an evidence for virtual proofs.
    /// A virtual proof for any NonFunbigleId can be created for any ResourceAddress in the map.
    /// Note: when a virtual proof is created,
    /// the resources aren't actually being added to the bucket.
    pub virtual_proofs_buckets: BTreeMap<ResourceAddress, BucketId>,
}

impl AuthZoneSubstate {
    pub fn new_with_proofs(
        proofs: Vec<ProofSubstate>,
        virtual_proofs_buckets: BTreeMap<ResourceAddress, BucketId>,
    ) -> Self {
        Self {
            proofs,
            virtual_proofs_buckets,
        }
    }

    pub fn new() -> Self {
        Self {
            proofs: Vec::new(),
            virtual_proofs_buckets: BTreeMap::new(),
        }
    }

    pub fn is_proof_virtualizable(&self, resource_address: &ResourceAddress) -> bool {
        self.virtual_proofs_buckets.contains_key(resource_address)
    }

    pub fn virtualize_non_fungible_proof(
        &self,
        resource_address: &ResourceAddress,
        ids: &BTreeSet<NonFungibleId>,
    ) -> ProofSubstate {
        let bucket_id = self
            .virtual_proofs_buckets
            .get(resource_address)
            .expect("Failed to create a virtual proof (bucket does not exist)")
            .clone();

        let mut locked_ids = BTreeMap::new();
        for id in ids.clone() {
            locked_ids.insert(id, 0);
        }
        let mut evidence = HashMap::new();
        evidence.insert(
            ResourceContainerId::Bucket(bucket_id),
            (
                Rc::new(RefCell::new(LockableResource::NonFungible {
                    resource_address: resource_address.clone(),
                    locked_ids: locked_ids,
                    liquid_ids: BTreeSet::new(),
                })),
                LockedAmountOrIds::Ids(ids.clone()),
            ),
        );
        ProofSubstate::new(
            resource_address.clone(),
            ResourceType::NonFungible,
            LockedAmountOrIds::Ids(ids.clone()),
            evidence,
        )
        .expect("Failed to create a virtual proof")
    }

    pub fn pop(&mut self) -> Result<ProofSubstate, InvokeError<AuthZoneError>> {
        if self.proofs.is_empty() {
            return Err(InvokeError::Error(AuthZoneError::EmptyAuthZone));
        }

        Ok(self.proofs.remove(self.proofs.len() - 1))
    }

    pub fn push(&mut self, proof: ProofSubstate) {
        self.proofs.push(proof);
    }

    pub fn drain(&mut self) -> Vec<ProofSubstate> {
        self.proofs.drain(0..).collect()
    }

    pub fn clear(&mut self) {
        loop {
            if let Some(proof) = self.proofs.pop() {
                proof.drop();
            } else {
                break;
            }
        }
    }

    pub fn create_proof(
        &self,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<ProofSubstate, InvokeError<AuthZoneError>> {
        ProofSubstate::compose(&self.proofs, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }

    pub fn create_proof_by_amount(
        &self,
        amount: Decimal,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<ProofSubstate, InvokeError<AuthZoneError>> {
        ProofSubstate::compose_by_amount(&self.proofs, amount, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }

    pub fn create_proof_by_ids(
        &self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<ProofSubstate, InvokeError<AuthZoneError>> {
        ProofSubstate::compose_by_ids(&self.proofs, ids, resource_address, resource_type)
            .map_err(|e| InvokeError::Error(AuthZoneError::ProofError(e)))
    }
}
