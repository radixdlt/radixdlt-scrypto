use scrypto::engine::types::*;
use scrypto::rust::cell::RefCell;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;
use scrypto::rust::vec::Vec;

use crate::model::{LockedAmountOrIds, ResourceContainer, ResourceContainerError};

#[derive(Debug)]
pub struct Proof {
    /// The resource definition id.
    resource_def_id: ResourceDefId,
    /// The resource type.
    resource_type: ResourceType,
    /// Whether movement of this proof is restricted.
    restricted: bool,
    /// The locked amounts or non-fungible ids of the resource.
    locked_amount_or_ids:
        HashMap<ProofSourceId, (Rc<RefCell<ResourceContainer>>, LockedAmountOrIds)>,
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
    /// Can't apply a fungible operation on non-fungible proofs.
    FungibleOperationNotAllowed,
}

impl Proof {
    pub fn new(
        resource_def_id: ResourceDefId,
        resource_type: ResourceType,
        restricted: bool,
        locked_amount_or_ids: HashMap<
            ProofSourceId,
            (Rc<RefCell<ResourceContainer>>, LockedAmountOrIds),
        >,
    ) -> Result<Proof, ProofError> {
        let proofs = vec![Self {
            resource_def_id,
            resource_type,
            restricted,
            locked_amount_or_ids,
        }];

        if Self::compute_max_locked(&proofs, resource_def_id, resource_type).is_empty() {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(proofs.into_iter().next().unwrap())
    }

    /// Computes the max amount or IDs of locked resource.
    pub fn compute_max_locked(
        proofs: &[Proof],
        resource_def_id: ResourceDefId,
        resource_type: ResourceType,
    ) -> LockedAmountOrIds {
        // filter proofs by resource def id and restricted flag
        let proofs: Vec<&Proof> = proofs
            .iter()
            .filter(|p| p.resource_def_id() == resource_def_id && !p.is_restricted())
            .collect();

        // calculate the max locked amount (or ids) in each container
        match resource_type {
            ResourceType::Fungible { .. } => {
                let mut max = HashMap::<ProofSourceId, Decimal>::new();
                for proof in &proofs {
                    for (source_id, (_, locked_amount_or_ids)) in &proof.locked_amount_or_ids {
                        let new_amount = locked_amount_or_ids.amount();
                        if let Some(existing) = max.get_mut(&source_id) {
                            *existing = Decimal::max(*existing, new_amount);
                        } else {
                            max.insert(source_id.clone(), new_amount);
                        }
                    }
                }
                let max_sum = max
                    .values()
                    .cloned()
                    .reduce(|a, b| a + b)
                    .unwrap_or_default();
                LockedAmountOrIds::Amount(max_sum)
            }
            ResourceType::NonFungible => {
                let mut max = HashMap::<ProofSourceId, BTreeSet<NonFungibleId>>::new();
                for proof in &proofs {
                    for (source_id, (_, locked_amount_or_ids)) in &proof.locked_amount_or_ids {
                        let new_ids = locked_amount_or_ids.ids();
                        if let Some(ids) = max.get_mut(&source_id) {
                            ids.extend(new_ids);
                        } else {
                            max.insert(source_id.clone(), new_ids);
                        }
                    }
                }
                let mut max_sum = BTreeSet::<NonFungibleId>::new();
                for (_, value) in max {
                    max_sum.extend(value);
                }
                LockedAmountOrIds::Ids(max_sum)
            }
        }
    }

    /// Creates a composite proof from proofs. This method will generate a max proof.
    pub fn compose(
        proofs: &[Proof],
        resource_def_id: ResourceDefId,
        resource_type: ResourceType,
    ) -> Result<Proof, ProofError> {
        let max = Self::compute_max_locked(proofs, resource_def_id, resource_type);
        match max {
            LockedAmountOrIds::Amount(amount) => {
                Self::compose_by_amount(proofs, amount, resource_def_id, resource_type)
            }
            LockedAmountOrIds::Ids(ids) => {
                Self::compose_by_ids(proofs, &ids, resource_def_id, resource_type)
            }
        }
    }

    pub fn compose_by_amount(
        proofs: &[Proof],
        amount: Decimal,
        resource_def_id: ResourceDefId,
        resource_type: ResourceType,
    ) -> Result<Proof, ProofError> {
        todo!("Re-implement")
    }

    pub fn compose_by_ids(
        proofs: &[Proof],
        ids: &BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
        resource_type: ResourceType,
    ) -> Result<Proof, ProofError> {
        todo!("Re-implement")
    }

    /// Makes a clone of this proof.
    ///
    /// Note that cloning a proof will update the ref count of the locked
    /// resources in the source containers.
    pub fn clone(&self) -> Self {
        for (_, (container, locked_amount_or_ids)) in &self.locked_amount_or_ids {
            match locked_amount_or_ids {
                LockedAmountOrIds::Amount(amount) => {
                    container
                        .borrow_mut()
                        .lock_by_amount(*amount)
                        .expect("Cloning should always succeed");
                }
                LockedAmountOrIds::Ids(ids) => {
                    container
                        .borrow_mut()
                        .lock_by_ids(ids)
                        .expect("Cloning should always succeed");
                }
            }
        }
        Self {
            resource_def_id: self.resource_def_id.clone(),
            resource_type: self.resource_type.clone(),
            restricted: self.restricted,
            locked_amount_or_ids: self.locked_amount_or_ids.clone(),
        }
    }

    pub fn drop(self) {
        for (_, (container, locked_amount_or_ids)) in self.locked_amount_or_ids {
            container.borrow_mut().unlock(locked_amount_or_ids);
        }
    }

    pub fn change_to_restricted(&mut self) {
        self.restricted = true;
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.resource_def_id
    }

    pub fn total_amount(&self) -> Decimal {
        todo!()
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ProofError> {
        todo!()
    }

    pub fn is_restricted(&self) -> bool {
        self.restricted
    }
}
