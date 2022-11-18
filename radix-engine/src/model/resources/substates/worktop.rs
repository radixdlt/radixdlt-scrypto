use crate::model::{
    BucketSubstate, LockableResource, Resource, ResourceOperationError, WorktopError,
};
use crate::types::*;

/// Worktop collects resources from function or method returns.
#[derive(Debug)]
pub struct WorktopSubstate {
    // TODO: refactor worktop to be `HashMap<ResourceAddress, BucketId>`
    pub resources: HashMap<ResourceAddress, Rc<RefCell<LockableResource>>>,
}

impl WorktopSubstate {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn drop(&mut self) -> Result<(), WorktopError> {
        for (_address, resource) in &self.resources {
            if !resource.borrow().is_empty() {
                return Err(WorktopError::CouldNotDrop);
            }
        }

        Ok(())
    }

    pub fn put(&mut self, other: BucketSubstate) -> Result<(), ResourceOperationError> {
        let resource_address = other.resource_address();
        let other_resource = other.resource()?;
        if let Some(mut resource) = self.borrow_resource_mut(resource_address) {
            return resource.put(other_resource);
        }
        self.resources.insert(
            resource_address,
            Rc::new(RefCell::new(other_resource.into())),
        );
        Ok(())
    }

    pub fn take(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> Result<Option<Resource>, ResourceOperationError> {
        if let Some(mut resource) = self.borrow_resource_mut(resource_address) {
            resource.take_by_amount(amount).map(Option::Some)
        } else if !amount.is_zero() {
            Err(ResourceOperationError::InsufficientBalance)
        } else {
            Ok(None)
        }
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Result<Option<Resource>, ResourceOperationError> {
        if let Some(mut resource) = self.borrow_resource_mut(resource_address) {
            resource.take_by_ids(ids).map(Option::Some)
        } else if !ids.is_empty() {
            Err(ResourceOperationError::InsufficientBalance)
        } else {
            Ok(None)
        }
    }

    pub fn take_all(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Result<Option<Resource>, ResourceOperationError> {
        if let Some(mut resource) = self.borrow_resource_mut(resource_address) {
            Ok(Some(resource.take_all_liquid()?))
        } else {
            Ok(None)
        }
    }

    pub fn resource_addresses(&self) -> Vec<ResourceAddress> {
        self.resources.keys().cloned().collect()
    }

    pub fn total_amount(&self, resource_address: ResourceAddress) -> Decimal {
        if let Some(resource) = self.borrow_resource(resource_address) {
            resource.total_amount()
        } else {
            Decimal::zero()
        }
    }

    pub fn total_ids(
        &self,
        resource_address: ResourceAddress,
    ) -> Result<BTreeSet<NonFungibleId>, ResourceOperationError> {
        if let Some(resource) = self.borrow_resource(resource_address) {
            resource.total_ids()
        } else {
            Ok(BTreeSet::new())
        }
    }

    pub fn is_locked(&self) -> bool {
        for resource_address in self.resource_addresses() {
            if let Some(resource) = self.borrow_resource(resource_address) {
                if resource.is_locked() {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        for resource_address in self.resource_addresses() {
            if let Some(resource) = self.borrow_resource(resource_address) {
                if !resource.total_amount().is_zero() {
                    return false;
                }
            }
        }
        true
    }

    pub fn create_reference_for_proof(
        &self,
        resource_address: ResourceAddress,
    ) -> Option<Rc<RefCell<LockableResource>>> {
        self.resources.get(&resource_address).map(Clone::clone)
    }

    pub fn borrow_resource(
        &self,
        resource_address: ResourceAddress,
    ) -> Option<Ref<LockableResource>> {
        self.resources.get(&resource_address).map(|c| c.borrow())
    }

    pub fn borrow_resource_mut(
        &mut self,
        resource_address: ResourceAddress,
    ) -> Option<RefMut<LockableResource>> {
        self.resources
            .get(&resource_address)
            .map(|c| c.borrow_mut())
    }
}
