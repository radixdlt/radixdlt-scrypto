use scrypto::engine::types::*;
use scrypto::rust::cell::{Ref, RefCell, RefMut};
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;

use crate::model::{Proof, ProofError, ProofSourceId, ResourceContainer, ResourceContainerError};

/// A transient resource container.
#[derive(Debug)]
pub struct Bucket {
    container: Rc<RefCell<ResourceContainer>>,
}

impl Bucket {
    pub fn new(container: ResourceContainer) -> Self {
        Self {
            container: Rc::new(RefCell::new(container)),
        }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), ResourceContainerError> {
        self.borrow_container_mut().put(other.into_container()?)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Bucket, ResourceContainerError> {
        Ok(Bucket::new(
            self.borrow_container_mut().take_by_amount(amount)?,
        ))
    }

    pub fn take_non_fungible(
        &mut self,
        id: &NonFungibleId,
    ) -> Result<Bucket, ResourceContainerError> {
        self.take_non_fungibles(&BTreeSet::from([id.clone()]))
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<Bucket, ResourceContainerError> {
        Ok(Bucket::new(self.borrow_container_mut().take_by_ids(ids)?))
    }

    pub fn create_proof(&mut self, proof_source_id: ProofSourceId) -> Result<Proof, ProofError> {
        match self.resource_type() {
            ResourceType::Fungible { .. } => {
                self.create_proof_by_amount(self.total_amount(), proof_source_id)
            }
            ResourceType::NonFungible => {
                self.create_proof_by_ids(&self.total_ids().unwrap(), proof_source_id)
            }
        }
    }

    pub fn create_proof_by_amount(
        &mut self,
        amount: Decimal,
        proof_source_id: ProofSourceId,
    ) -> Result<Proof, ProofError> {
        // lock the specified amount
        let locked_amount_or_ids = self
            .borrow_container_mut()
            .lock_by_amount(amount)
            .map_err(ProofError::ResourceContainerError)?;

        // produce proof
        let mut map = HashMap::new();
        map.insert(
            proof_source_id,
            (self.container.clone(), locked_amount_or_ids),
        );
        Proof::new(self.resource_def_id(), self.resource_type(), false, map)
    }

    pub fn create_proof_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        proof_source_id: ProofSourceId,
    ) -> Result<Proof, ProofError> {
        // lock the specified id set
        let locked_amount_or_ids = self
            .borrow_container_mut()
            .lock_by_ids(ids)
            .map_err(ProofError::ResourceContainerError)?;

        // produce proof
        let mut map = HashMap::new();
        map.insert(
            proof_source_id,
            (self.container.clone(), locked_amount_or_ids),
        );
        Proof::new(self.resource_def_id(), self.resource_type(), false, map)
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.borrow_container().resource_def_id()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.borrow_container().resource_type()
    }

    pub fn total_amount(&self) -> Decimal {
        self.borrow_container().total_amount()
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ResourceContainerError> {
        self.borrow_container().total_ids()
    }

    pub fn is_locked(&self) -> bool {
        self.borrow_container().is_locked()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow_container().is_empty()
    }

    pub fn into_container(self) -> Result<ResourceContainer, ResourceContainerError> {
        Rc::try_unwrap(self.container)
            .map_err(|_| ResourceContainerError::ContainerLocked)
            .map(|c| c.into_inner())
    }

    fn borrow_container(&self) -> Ref<ResourceContainer> {
        self.container.borrow()
    }

    fn borrow_container_mut(&mut self) -> RefMut<ResourceContainer> {
        self.container.borrow_mut()
    }
}
