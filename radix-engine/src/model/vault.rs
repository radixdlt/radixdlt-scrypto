use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::cell::{Ref, RefCell, RefMut};
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;

use crate::model::{
    Bucket, Proof, ProofError, ResourceContainer, ResourceContainerError, ResourceContainerId,
};
/// A persistent resource container.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct Vault {
    container: Rc<RefCell<ResourceContainer>>,
}

impl Vault {
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

    pub fn create_proof(&mut self, container_id: ResourceContainerId) -> Result<Proof, ProofError> {
        match self.resource_type() {
            ResourceType::Fungible { .. } => {
                self.create_proof_by_amount(self.total_amount(), container_id)
            }
            ResourceType::NonFungible => {
                self.create_proof_by_ids(&self.total_ids().unwrap(), container_id)
            }
        }
    }

    pub fn create_proof_by_amount(
        &mut self,
        amount: Decimal,
        container_id: ResourceContainerId,
    ) -> Result<Proof, ProofError> {
        // lock the specified amount
        let locked_amount_or_ids = self
            .borrow_container_mut()
            .lock_by_amount(amount)
            .map_err(ProofError::ResourceContainerError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.container.clone(), locked_amount_or_ids.clone()),
        );
        Proof::new(
            self.resource_address(),
            self.resource_type(),
            false,
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn create_proof_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        container_id: ResourceContainerId,
    ) -> Result<Proof, ProofError> {
        // lock the specified id set
        let locked_amount_or_ids = self
            .borrow_container_mut()
            .lock_by_ids(ids)
            .map_err(ProofError::ResourceContainerError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.container.clone(), locked_amount_or_ids.clone()),
        );
        Proof::new(
            self.resource_address(),
            self.resource_type(),
            false,
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.borrow_container().resource_address()
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

    fn borrow_container(&self) -> Ref<ResourceContainer> {
        self.container.borrow()
    }

    fn borrow_container_mut(&mut self) -> RefMut<ResourceContainer> {
        self.container.borrow_mut()
    }
}
