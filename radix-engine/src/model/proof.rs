use scrypto::engine::types::*;
use scrypto::rust::cell::RefCell;
use scrypto::rust::rc::Rc;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;

use crate::model::{ResourceContainer, ResourceContainerError};

#[derive(Debug)]
pub struct Proof {
    /// The resource definition id
    resource_def_id: ResourceDefId,
    /// The resource type
    resource_type: ResourceType,
    /// Restricted proof can't be moved down along the call stack (growing down).
    restricted: bool,
    /// The total amount for optimization purpose
    total_amount: Amount,
    /// The containers that supports this proof
    supporting_containers: Vec<(Rc<RefCell<ResourceContainer>>, Amount)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofError {
    SupportContainerError(ResourceContainerError),
    ZeroAmountProofNotAllowed,
}

impl Proof {
    // TODO: partial proof
    // TODO: multiple containers
    // TODO: mixed types of container
    // TODO: restricted proof
    // TODO: proof auto drop

    pub fn new(container: Rc<RefCell<ResourceContainer>>) -> Result<Self, ProofError> {
        let resource_def_id = container.borrow().resource_def_id();
        let resource_type = container.borrow().resource_type();

        // lock the full amount
        let total_amount = container.borrow().total_amount();
        if total_amount.is_zero() {
            return Err(ProofError::ZeroAmountProofNotAllowed);
        }
        container
            .borrow_mut()
            .lock(&total_amount)
            .map_err(ProofError::SupportContainerError)?;

        // record the supporting container
        let supporting_containers = vec![(container, total_amount.clone())];

        // generate proof
        Ok(Self {
            resource_def_id,
            resource_type,
            restricted: false,
            total_amount,
            supporting_containers,
        })
    }

    pub fn clone(&self) -> Self {
        for (container, amount) in &self.supporting_containers {
            container
                .borrow_mut()
                .lock(amount)
                .expect("Cloning should be always possible");
        }

        Self {
            resource_def_id: self.resource_def_id,
            resource_type: self.resource_type,
            restricted: self.restricted,
            total_amount: self.total_amount.clone(),
            supporting_containers: self.supporting_containers.clone(),
        }
    }

    pub fn settle(self) {
        for (container, amount) in self.supporting_containers {
            container
                .borrow_mut()
                .unlock(&amount)
                .expect("Unlocking should be always possible");
        }
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }

    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    pub fn total_amount(&self) -> Amount {
        self.total_amount.clone()
    }

    pub fn is_restricted(&self) -> bool {
        self.restricted
    }
}
