use scrypto::engine::types::*;
use scrypto::resource::NonFungibleId;
use scrypto::rust::cell::RefCell;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;
use scrypto::rust::vec::Vec;

use crate::model::{ResourceContainer, ResourceContainerError};

#[derive(Debug, Clone)]
pub enum AmountOrIds {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleId>),
}

impl AmountOrIds {
    pub fn as_amount(&self) -> Decimal {
        match self {
            Self::Amount(amount) => amount.clone(),
            Self::Ids(ids) => ids.len().into(),
        }
    }

    pub fn as_ids(&self) -> Result<BTreeSet<NonFungibleId>, ProofError> {
        match self {
            Self::Amount(_) => Err(ProofError::NonFungibleOperationNotAllowed),
            Self::Ids(ids) => Ok(ids.clone()),
        }
    }
}

#[derive(Debug)]
pub struct Proof {
    /// The resource definition id.
    resource_def_id: ResourceDefId,
    /// Restricted proof can't be moved.
    restricted: bool,
    /// The total amount, for optimization purpose.
    locked_total: AmountOrIds,
    /// The containers that supports this proof.
    locked_details: Vec<(Rc<RefCell<ResourceContainer>>, ProofSourceId, AmountOrIds)>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ProofSourceId {
    Bucket(BucketId),
    Vault(VaultId),
    Worktop(u32, ResourceDefId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofError {
    /// Error produced by a resource container.
    ResourceContainerError(ResourceContainerError),
    /// Can't generate zero-amount or empty non-fungible set proofs.
    EmptyProofNotAllowed,
    /// The base proofs are not enough to cover the requested amount or non-fungible ids.
    InsufficientBaseProofs,
    /// Can't apply a non-fungible operation on fungible proofs.
    NonFungibleOperationNotAllowed,
}

impl Proof {
    // TODO: proof auto drop
    // TODO: thorough test partial/full/composite proofs

    pub fn new(
        resource_def_id: ResourceDefId,
        restricted: bool,
        locked_total: AmountOrIds,
        locked_details: Vec<(Rc<RefCell<ResourceContainer>>, ProofSourceId, AmountOrIds)>,
    ) -> Proof {
        Self {
            resource_def_id,
            restricted,
            locked_total,
            locked_details,
        }
    }

    pub fn create_proof_by_amount(
        proofs: &[Proof],
        amount: Decimal,
        resource_def_id: ResourceDefId,
    ) -> Result<Proof, ProofError> {
        // calculate the max locked amount (by the input proofs) in each container
        let mut allowance = HashMap::<ProofSourceId, Decimal>::new();
        for proof in proofs {
            if proof.resource_def_id == resource_def_id && !proof.is_restricted() {
                for (_, source_id, amount_or_ids) in &proof.locked_details {
                    if let Some(amount) = allowance.get_mut(source_id) {
                        *amount = Decimal::max(*amount, amount_or_ids.as_amount());
                    } else {
                        allowance.insert(source_id.clone(), amount_or_ids.as_amount());
                    }
                }
            }
        }

        // check if the allowance satisfied the requested amount
        let max = allowance
            .values()
            .cloned()
            .reduce(|a, b| a + b)
            .unwrap_or_default();
        if amount > max {
            return Err(ProofError::InsufficientBaseProofs);
        }

        // lock all relevant containers
        //
        // This is not an efficient way of producing proofs, in terms of number of state updates
        // to the resource containers. However, this is the simplest to explain as no
        // resource container selection algorithm is required. All the ref count increases by 1.
        //
        // If this turns to be performance bottleneck, should start with containers where the
        // largest amount has been locked, and only lock the requested amount.
        //
        let mut locked_details = Vec::new();
        for proof in proofs {
            if proof.resource_def_id == resource_def_id {
                for entry in &proof.locked_details {
                    entry
                        .0
                        .borrow_mut()
                        .lock_amount(entry.2.as_amount())
                        .map_err(ProofError::ResourceContainerError)
                        .expect("Should always be able to lock the same amount");
                    locked_details.push(entry.clone());
                }
            }
        }

        // issue a new proof
        Ok(Proof::new(
            resource_def_id,
            false,
            AmountOrIds::Amount(amount),
            locked_details,
        ))
    }

    pub fn compose_by_ids(
        proofs: &[Proof],
        ids: BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    ) -> Result<Proof, ProofError> {
        // calculate the max locked amount (by the input proofs) in each container
        let mut allowance = HashMap::<ProofSourceId, BTreeSet<NonFungibleId>>::new();
        for proof in proofs {
            if proof.resource_def_id == resource_def_id && !proof.is_restricted() {
                for (_, source_id, amount_or_ids) in &proof.locked_details {
                    if let Some(ids) = allowance.get_mut(source_id) {
                        ids.extend(amount_or_ids.as_ids()?);
                    } else {
                        allowance.insert(source_id.clone(), amount_or_ids.as_ids()?);
                    }
                }
            }
        }

        // check if the allowance satisfied the requested amount
        let mut max = BTreeSet::<NonFungibleId>::new();
        for (_, value) in allowance {
            max.extend(value);
        }
        if !max.is_superset(&ids) {
            return Err(ProofError::InsufficientBaseProofs);
        }

        // lock all relevant resources
        //
        // See `compose_by_amount` for performance notes.
        //
        let mut locked_details = Vec::new();
        for proof in proofs {
            if proof.resource_def_id == resource_def_id {
                for entry in &proof.locked_details {
                    entry
                        .0
                        .borrow_mut()
                        .lock_ids(&entry.2.as_ids()?)
                        .map_err(ProofError::ResourceContainerError)
                        .expect("Should always be able to lock the same non-fungibles");
                    locked_details.push(entry.clone());
                }
            }
        }

        // issue a new proof
        Ok(Proof::new(
            resource_def_id,
            false,
            AmountOrIds::Ids(ids),
            locked_details,
        ))
    }

    pub fn clone(&self) -> Self {
        for (container, _, amount_or_ids) in &self.locked_details {
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
            restricted: self.restricted,
            locked_total: self.locked_total.clone(),
            locked_details: self.locked_details.clone(),
        }
    }

    pub fn drop(self) {
        for (container, _, amount_or_ids) in self.locked_details {
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
