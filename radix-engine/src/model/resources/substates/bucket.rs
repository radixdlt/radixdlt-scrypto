use crate::model::{
    LockableResource, ProofError, ProofSubstate, Resource, ResourceContainerId,
    ResourceOperationError,
};
use crate::types::*;
use radix_engine_interface::api::types::BucketId;
use radix_engine_interface::model::*;

/// A transient resource container.
#[derive(Debug)]
pub struct BucketSubstate {
    resource: Rc<RefCell<LockableResource>>,
}

impl BucketSubstate {
    pub fn new(resource: Resource) -> Self {
        Self {
            resource: Rc::new(RefCell::new(resource.into())),
        }
    }

    pub fn put(&mut self, other: BucketSubstate) -> Result<(), ResourceOperationError> {
        self.borrow_resource_mut().put(other.resource()?)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Resource, ResourceOperationError> {
        self.borrow_resource_mut().take_by_amount(amount)
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> Result<Resource, ResourceOperationError> {
        self.borrow_resource_mut().take_by_ids(ids)
    }

    pub fn create_proof(&mut self, self_bucket_id: BucketId) -> Result<ProofSubstate, ProofError> {
        let container_id = ResourceContainerId::Bucket(self_bucket_id);
        match self.resource_type() {
            ResourceType::Fungible { .. } => {
                self.create_proof_by_amount(self.total_amount(), container_id)
            }
            ResourceType::NonFungible { .. } => self.create_proof_by_ids(
                &self
                    .total_ids()
                    .expect("Failed to list non-fungible IDs on non-fungible Bucket"),
                container_id,
            ),
        }
    }

    pub fn create_proof_by_amount(
        &mut self,
        amount: Decimal,
        container_id: ResourceContainerId,
    ) -> Result<ProofSubstate, ProofError> {
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
        ProofSubstate::new(
            self.resource_address(),
            self.resource_type(),
            locked_amount_or_ids,
            evidence,
        )
    }

    pub fn create_proof_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleLocalId>,
        container_id: ResourceContainerId,
    ) -> Result<ProofSubstate, ProofError> {
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
        ProofSubstate::new(
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

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleLocalId>, ResourceOperationError> {
        self.borrow_resource().total_ids()
    }

    pub fn is_locked(&self) -> bool {
        self.borrow_resource().is_locked()
    }

    pub fn is_empty(&self) -> bool {
        self.borrow_resource().is_empty()
    }

    pub fn resource(self) -> Result<Resource, ResourceOperationError> {
        Rc::try_unwrap(self.resource)
            .map_err(|_| ResourceOperationError::ResourceLocked)
            .map(|c| c.into_inner())
            .map(Into::into)
    }

    pub fn borrow_resource(&self) -> Ref<LockableResource> {
        self.resource.borrow()
    }

    pub fn borrow_resource_mut(&mut self) -> RefMut<LockableResource> {
        self.resource.borrow_mut()
    }

    pub fn peek_resource(&self) -> Resource {
        let lockable_resource: &LockableResource = &self.borrow_resource();
        lockable_resource.peek_resource()
    }
}
