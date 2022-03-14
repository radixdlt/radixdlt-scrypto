use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;

use crate::model::{ResourceAmount, ResourceContainer, ResourceError};

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum BucketError {
    ResourceError(ResourceError),
    BucketLocked,
    OtherBucketLocked,
}

/// A transient resource container.
#[derive(Debug)]
pub struct Bucket {
    container: Rc<ResourceContainer>,
}

impl Bucket {
    pub fn new(container: ResourceContainer) -> Self {
        Self {
            container: Rc::new(container),
        }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), BucketError> {
        let this_container = self.borrow_container()?;
        let other_container = other
            .own_container()
            .map_err(|_| BucketError::OtherBucketLocked)?;

        this_container
            .put(other_container)
            .map_err(BucketError::ResourceError)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Bucket, BucketError> {
        let this_container = self.borrow_container()?;

        Ok(Bucket::new(
            this_container
                .take(amount)
                .map_err(BucketError::ResourceError)?,
        ))
    }

    pub fn take_non_fungible(&mut self, key: &NonFungibleId) -> Result<Bucket, BucketError> {
        self.take_non_fungibles(&BTreeSet::from([key.clone()]))
    }

    pub fn take_non_fungibles(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
    ) -> Result<Bucket, BucketError> {
        let this_container = self.borrow_container()?;

        Ok(Bucket::new(
            this_container
                .take_non_fungibles(ids)
                .map_err(BucketError::ResourceError)?,
        ))
    }

    pub fn liquid_amount(&self) -> ResourceAmount {
        self.container.liquid_amount()
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.container.resource_def_id()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.container.resource_type()
    }

    /// Creates another `Rc<ResourceContainer>` to the container
    pub fn reference_container(&self) -> Rc<ResourceContainer> {
        self.container.clone()
    }

    /// Creates a mutable reference to the container
    pub fn borrow_container(&mut self) -> Result<&mut ResourceContainer, BucketError> {
        Ok(Rc::get_mut(&mut self.container).ok_or(BucketError::BucketLocked)?)
    }

    /// Takes the ownership of the container
    pub fn own_container(self) -> Result<ResourceContainer, BucketError> {
        Ok(Rc::try_unwrap(self.container).map_err(|_| BucketError::BucketLocked)?)
    }
}
