use scrypto::engine::types::*;
use scrypto::rust::cell::{Ref, RefCell, RefMut};
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;
use scrypto::rust::vec;

use crate::model::{AmountOrIds, Proof, ProofSourceId, ResourceContainer, ResourceContainerError};

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
        Ok(Bucket::new(self.borrow_container_mut().take(amount)?))
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
        Ok(Bucket::new(
            self.borrow_container_mut().take_non_fungibles(ids)?,
        ))
    }

    pub fn create_proof(
        &mut self,
        proof_source_id: ProofSourceId,
    ) -> Result<Proof, ResourceContainerError> {
        match self.resource_type() {
            ResourceType::Fungible { .. } => {
                self.create_proof_by_amount(self.total_amount(), proof_source_id)
            }
            ResourceType::NonFungible => {
                self.create_proof_by_ids(&self.total_ids()?, proof_source_id)
            }
        }
    }

    pub fn create_proof_by_amount(
        &mut self,
        amount: Decimal,
        proof_source_id: ProofSourceId,
    ) -> Result<Proof, ResourceContainerError> {
        // do not allow empty proof
        if amount.is_zero() {
            return Err(ResourceContainerError::CantCreateEmptyProof);
        }

        // lock the specified amount
        self.borrow_container_mut().lock_amount(amount)?;

        // produce proof
        Ok(Proof::new(
            self.resource_def_id(),
            self.resource_type(),
            false,
            AmountOrIds::Amount(amount),
            vec![(
                self.container.clone(),
                proof_source_id,
                AmountOrIds::Amount(amount),
            )],
        ))
    }

    pub fn create_proof_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        proof_source_id: ProofSourceId,
    ) -> Result<Proof, ResourceContainerError> {
        // do not allow empty proof
        if ids.is_empty() {
            return Err(ResourceContainerError::CantCreateEmptyProof);
        }

        // lock the specified id set
        self.borrow_container_mut().lock_ids(ids)?;

        // produce proof
        Ok(Proof::new(
            self.resource_def_id(),
            self.resource_type(),
            false,
            AmountOrIds::Ids(ids.clone()),
            vec![(
                self.container.clone(),
                proof_source_id,
                AmountOrIds::Ids(ids.clone()),
            )],
        ))
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
