use crate::engine::{REActor, ResolvedMethod, ResolvedReceiverMethod};
use crate::model::{
    AuthZoneError, InvokeError, LockableResource, LockedAmountOrIds, ProofSubstate,
    ResourceContainerId,
};
use crate::types::*;

/// A transient resource container.
#[derive(Debug)]
pub struct AuthZoneSubstate {
    pub auth_zones: Vec<AuthZone>,
}

impl AuthZoneSubstate {
    pub fn new_frame(&mut self, actor: &REActor) {
        if matches!(
            actor,
            REActor::Method(ResolvedReceiverMethod {
                method: ResolvedMethod::Native(NativeMethod::AuthZone(..)),
                ..
            })
        ) {
            return;
        }

        let virtual_proofs_buckets = self
            .auth_zones
            .first()
            .unwrap()
            .virtual_proofs_buckets
            .clone();
        self.auth_zones
            .push(AuthZone::new_with_proofs(vec![], virtual_proofs_buckets));
    }

    pub fn pop_frame(&mut self, actor: &REActor) {
        if matches!(
            actor,
            REActor::Method(ResolvedReceiverMethod {
                method: ResolvedMethod::Native(NativeMethod::AuthZone(..)),
                ..
            })
        ) {
            return;
        }

        if let Some(mut auth_zone) = self.auth_zones.pop() {
            auth_zone.clear()
        }
    }

    pub fn clear_all(&mut self) {
        for auth_zone in &mut self.auth_zones {
            auth_zone.clear()
        }
    }

    pub fn cur_auth_zone_mut(&mut self) -> &mut AuthZone {
        self.auth_zones.last_mut().unwrap()
    }

    pub fn cur_auth_zone(&mut self) -> &AuthZone {
        self.auth_zones.last().unwrap()
    }
}

#[derive(Debug)]
pub struct AuthZone {
    pub proofs: Vec<ProofSubstate>,
    /// IDs of buckets that act as an evidence for virtual proofs.
    /// A virtual proof for any NonFunbigleId can be created for any ResourceAddress in the map.
    /// Note: when a virtual proof is created,
    /// the resources aren't actually being added to the bucket.
    pub virtual_proofs_buckets: BTreeMap<ResourceAddress, BucketId>,
}

impl AuthZone {
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
