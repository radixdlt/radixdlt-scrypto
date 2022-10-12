use crate::model::{
    Bucket, InvokeError, LockableResource, Proof, ProofError, Resource, ResourceContainerId,
    ResourceOperationError, VaultError,
};
use crate::types::*;
use std::ops::Deref;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct VaultSubstate(pub Resource);

#[derive(Debug)]
pub struct VaultRuntimeSubstate {
    resource: Rc<RefCell<LockableResource>>,
}

impl VaultRuntimeSubstate {
    pub fn clone_to_persisted(&self) -> VaultSubstate {
        let lockable_resource = self.borrow_resource();
        if lockable_resource.is_locked() {
            // We keep resource containers in Rc<RefCell> for all concrete resource containers, like Bucket, Vault and Worktop.
            // When extracting the resource within a container, there should be no locked resource.
            // It should have failed the Rc::try_unwrap() check.
            panic!("Attempted to convert resource container with locked resource");
        }
        let resource = match lockable_resource.deref() {
            LockableResource::Fungible {
                resource_address,
                divisibility,
                liquid_amount,
                ..
            } => Resource::Fungible {
                resource_address: resource_address.clone(),
                divisibility: divisibility.clone(),
                amount: liquid_amount.clone(),
            },
            LockableResource::NonFungible {
                resource_address,
                liquid_ids,
                ..
            } => Resource::NonFungible {
                resource_address: resource_address.clone(),
                ids: liquid_ids.clone(),
            },
        };

        VaultSubstate(resource)
    }

    pub fn to_persisted(self) -> Result<VaultSubstate, ResourceOperationError> {
        Rc::try_unwrap(self.resource)
            .map_err(|_| ResourceOperationError::ResourceLocked)
            .map(|c| c.into_inner())
            .map(Into::into)
            .map(|r| VaultSubstate(r))
    }

    pub fn new(resource: Resource) -> Self {
        Self {
            resource: Rc::new(RefCell::new(resource.into())),
        }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), ResourceOperationError> {
        self.borrow_resource_mut().put(other.resource()?)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Resource, InvokeError<VaultError>> {
        let resource = self
            .borrow_resource_mut()
            .take_by_amount(amount)
            .map_err(|e| InvokeError::Error(VaultError::ResourceOperationError(e)))?;
        Ok(resource)
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<Resource, InvokeError<VaultError>> {
        let resource = self
            .borrow_resource_mut()
            .take_by_ids(ids)
            .map_err(|e| InvokeError::Error(VaultError::ResourceOperationError(e)))?;
        Ok(resource)
    }

    pub fn create_proof(&mut self, container_id: ResourceContainerId) -> Result<Proof, ProofError> {
        match self.resource_type() {
            ResourceType::Fungible { .. } => {
                self.create_proof_by_amount(self.total_amount(), container_id)
            }
            ResourceType::NonFungible => self.create_proof_by_ids(
                &self
                    .total_ids()
                    .expect("Failed to list non-fungible IDs of non-fungible vault"),
                container_id,
            ),
        }
    }

    pub fn create_proof_by_amount(
        &mut self,
        amount: Decimal,
        container_id: ResourceContainerId,
    ) -> Result<Proof, ProofError> {
        // lock the specified amount
        let locked_amount_or_ids = self
            .borrow_resource_mut()
            .lock_by_amount(amount)
            .map_err(ProofError::ResourceOperationError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.resource.clone(), locked_amount_or_ids.clone()),
        );
        Proof::new(
            self.resource_address(),
            self.resource_type(),
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
            .borrow_resource_mut()
            .lock_by_ids(ids)
            .map_err(ProofError::ResourceOperationError)?;

        // produce proof
        let mut evidence = HashMap::new();
        evidence.insert(
            container_id,
            (self.resource.clone(), locked_amount_or_ids.clone()),
        );
        Proof::new(
            self.resource_address(),
            self.resource_type(),
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.borrow_resource().resource_address()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.borrow_resource().resource_type()
    }

    pub fn total_amount(&self) -> Decimal {
        self.borrow_resource().total_amount()
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ResourceOperationError> {
        self.borrow_resource().total_ids()
    }

    pub fn is_locked(&self) -> bool {
        self.borrow_resource().is_locked()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow_resource().is_empty()
    }

    pub fn borrow_resource(&self) -> Ref<LockableResource> {
        self.resource.borrow()
    }

    pub fn borrow_resource_mut(&mut self) -> RefMut<LockableResource> {
        self.resource.borrow_mut()
    }
}
