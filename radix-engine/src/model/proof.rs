use scrypto::engine::types::*;
use scrypto::resource::NonFungibleId;
use scrypto::rust::cell::RefCell;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;

use crate::model::{ResourceContainer, ResourceContainerError};

#[derive(Debug, Clone)]
pub enum LockedAmountOrIds {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleId>),
}

#[derive(Debug)]
pub struct Proof {
    /// The resource definition id
    resource_def_id: ResourceDefId,
    /// The resource type
    resource_type: ResourceType,
    /// Restricted proof can't be moved down along the call stack (growing down).
    restricted: bool,
    /// The total amount, for optimization purpose
    locked_in_total: LockedAmountOrIds,
    /// The containers that supports this proof
    locked_in_details: Vec<(Rc<RefCell<ResourceContainer>>, LockedAmountOrIds)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofError {
    SupportContainerError(ResourceContainerError),
    ZeroAmountProofNotAllowed,
    NonFungibleOperationNotAllowed,
}

impl Proof {
    // TODO: partial proof
    // TODO: multiple containers
    // TODO: mixed types of container
    // TODO: restricted proof
    // TODO: proof auto drop
    // TODO: thorough test partial/full/composite proofs

    pub fn new(container: Rc<RefCell<ResourceContainer>>) -> Result<Self, ProofError> {
        let resource_def_id = container.borrow().resource_def_id();
        let resource_type = container.borrow().resource_type();

        // do not allow empty proof
        if container.borrow().is_empty() {
            return Err(ProofError::ZeroAmountProofNotAllowed);
        }

        // lock the full amount
        let locked_amount_or_ids = match &resource_type {
            ResourceType::Fungible { .. } => {
                let total_amount = container.borrow().total_amount();
                container
                    .borrow_mut()
                    .lock_amount(total_amount)
                    .expect("Should be able to lock the full amount");
                LockedAmountOrIds::Amount(total_amount)
            }
            ResourceType::NonFungible { .. } => {
                let total_ids = container
                    .borrow()
                    .total_ids()
                    .map_err(ProofError::SupportContainerError)?;
                container
                    .borrow_mut()
                    .lock_ids(&total_ids)
                    .expect("Should be able to lock the full id set");
                LockedAmountOrIds::Ids(total_ids)
            }
        };

        // record the locked amount or ids in detail
        let locked_in_details = vec![(container, locked_amount_or_ids.clone())];

        // generate proof
        Ok(Self {
            resource_def_id,
            resource_type,
            restricted: false,
            locked_in_total: locked_amount_or_ids,
            locked_in_details,
        })
    }

    pub fn clone(&self) -> Self {
        for (container, amount_or_ids) in &self.locked_in_details {
            match amount_or_ids {
                LockedAmountOrIds::Amount(amount) => container
                    .borrow_mut()
                    .lock_amount(amount.clone())
                    .expect("Cloning should always be possible"),
                LockedAmountOrIds::Ids(ids) => container
                    .borrow_mut()
                    .lock_ids(ids)
                    .expect("Cloning should always be possible"),
            };
        }

        Self {
            resource_def_id: self.resource_def_id,
            resource_type: self.resource_type,
            restricted: self.restricted,
            locked_in_total: self.locked_in_total.clone(),
            locked_in_details: self.locked_in_details.clone(),
        }
    }

    pub fn settle(self) {
        for (container, amount_or_ids) in self.locked_in_details {
            match amount_or_ids {
                LockedAmountOrIds::Amount(amount) => container
                    .borrow_mut()
                    .unlock_amount(amount)
                    .expect("Unlocking should always be possible"),
                LockedAmountOrIds::Ids(ids) => container
                    .borrow_mut()
                    .unlock_ids(&ids)
                    .expect("Unlocking should always be possible"),
            };
        }
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }

    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    pub fn total_amount(&self) -> Decimal {
        match &self.locked_in_total {
            LockedAmountOrIds::Amount(amount) => amount.clone(),
            LockedAmountOrIds::Ids(ids) => ids.len().into(),
        }
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ProofError> {
        match &self.locked_in_total {
            LockedAmountOrIds::Amount(_) => Err(ProofError::NonFungibleOperationNotAllowed),
            LockedAmountOrIds::Ids(ids) => Ok(ids.clone()),
        }
    }

    pub fn is_restricted(&self) -> bool {
        self.restricted
    }
}
