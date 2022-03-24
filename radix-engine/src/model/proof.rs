use scrypto::engine::types::*;
use scrypto::resource::NonFungibleId;
use scrypto::rust::cell::RefCell;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::rc::Rc;
use scrypto::rust::vec::Vec;

use crate::model::{ResourceContainer, ResourceContainerError};

#[derive(Debug, Clone)]
pub enum AmountOrIds {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleId>),
}

#[derive(Debug)]
pub struct Proof {
    /// The resource definition id.
    resource_def_id: ResourceDefId,
    /// The resource type.
    resource_type: ResourceType,
    /// Restricted proof can't be moved.
    restricted: bool,
    /// The total amount, for optimization purpose.
    locked_total: AmountOrIds,
    /// The containers that supports this proof.
    locked_details: Vec<(Rc<RefCell<ResourceContainer>>, AmountOrIds)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofError {
    /// Error produced by a resource container.
    ResourceContainerError(ResourceContainerError),
    /// Can't generate zero-amount or empty non-fungible set proofs.
    EmptyProofNotAllowed,
    /// Can't apply a non-fungible operation on fungible proofs.
    NonFungibleOperationNotAllowed,
}

impl Proof {
    // TODO: composite proofs
    // TODO: proof auto drop
    // TODO: thorough test partial/full/composite proofs

    pub fn new(
        resource_def_id: ResourceDefId,
        resource_type: ResourceType,
        restricted: bool,
        locked_total: AmountOrIds,
        locked_details: Vec<(Rc<RefCell<ResourceContainer>>, AmountOrIds)>,
    ) -> Result<Proof, ProofError> {
        if match &locked_total {
            AmountOrIds::Amount(amount) => amount.is_zero(),
            AmountOrIds::Ids(ids) => ids.is_empty(),
        } {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(Self {
            resource_def_id,
            resource_type,
            restricted,
            locked_total,
            locked_details,
        })
    }

    pub fn clone(&self) -> Self {
        for (container, amount_or_ids) in &self.locked_details {
            match amount_or_ids {
                AmountOrIds::Amount(amount) => container
                    .borrow_mut()
                    .lock_amount(amount.clone())
                    .expect("Cloning should always be possible"),
                AmountOrIds::Ids(ids) => container
                    .borrow_mut()
                    .lock_ids(ids)
                    .expect("Cloning should always be possible"),
            };
        }

        Self {
            resource_def_id: self.resource_def_id,
            resource_type: self.resource_type,
            restricted: self.restricted,
            locked_total: self.locked_total.clone(),
            locked_details: self.locked_details.clone(),
        }
    }

    pub fn drop(self) {
        for (container, amount_or_ids) in self.locked_details {
            match amount_or_ids {
                AmountOrIds::Amount(amount) => container
                    .borrow_mut()
                    .unlock_amount(amount)
                    .expect("Unlocking should always be possible"),
                AmountOrIds::Ids(ids) => container
                    .borrow_mut()
                    .unlock_ids(&ids)
                    .expect("Unlocking should always be possible"),
            };
        }
    }

    pub fn change_to_restricted(&mut self) {
        self.restricted = true;
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }

    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    pub fn total_amount(&self) -> Decimal {
        match &self.locked_total {
            AmountOrIds::Amount(amount) => amount.clone(),
            AmountOrIds::Ids(ids) => ids.len().into(),
        }
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ProofError> {
        match &self.locked_total {
            AmountOrIds::Amount(_) => Err(ProofError::NonFungibleOperationNotAllowed),
            AmountOrIds::Ids(ids) => Ok(ids.clone()),
        }
    }

    pub fn is_restricted(&self) -> bool {
        self.restricted
    }
}
