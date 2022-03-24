use scrypto::engine::types::*;
use scrypto::resource::NonFungibleId;
use scrypto::rust::cell::RefCell;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::HashMap;
use scrypto::rust::rc::Rc;
use scrypto::rust::vec::Vec;

use crate::model::{ResourceContainer, ResourceContainerError};

#[derive(Debug)]
pub enum Proof {
    Fungible {
        /// The resource definition id.
        resource_def_id: ResourceDefId,
        /// Restricted proof can't be moved.
        restricted: bool,
        /// The total amount this proof proves
        total_amount: Decimal,
        /// The proof sources (the sum of which may exceed total amount)
        sources: HashMap<ProofSourceId, (Rc<RefCell<ResourceContainer>>, Decimal)>,
    },
    NonFungible {
        /// The resource definition id.
        resource_def_id: ResourceDefId,
        /// Restricted proof can't be moved.
        restricted: bool,
        /// The total non-fungible IDs this proof proves
        total_ids: BTreeSet<NonFungibleId>,
        /// The proof sources (the sum of which may exceed total ids)
        sources: HashMap<ProofSourceId, (Rc<RefCell<ResourceContainer>>, BTreeSet<NonFungibleId>)>,
    },
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
    pub fn new_fungible(
        resource_def_id: ResourceDefId,
        restricted: bool,
        total_amount: Decimal,
        sources: HashMap<ProofSourceId, (Rc<RefCell<ResourceContainer>>, Decimal)>,
    ) -> Result<Proof, ProofError> {
        if total_amount.is_zero() {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(Self::Fungible {
            resource_def_id,
            restricted,
            total_amount,
            sources,
        })
    }

    pub fn new_non_fungible(
        resource_def_id: ResourceDefId,
        restricted: bool,
        total_ids: BTreeSet<NonFungibleId>,
        sources: HashMap<ProofSourceId, (Rc<RefCell<ResourceContainer>>, BTreeSet<NonFungibleId>)>,
    ) -> Result<Proof, ProofError> {
        if total_ids.is_empty() {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(Self::NonFungible {
            resource_def_id,
            restricted,
            total_ids,
            sources,
        })
    }

    pub fn compose(
        proofs: &[Proof],
        resource_def_id: ResourceDefId,
        resource_type: ResourceType,
    ) -> Result<Proof, ProofError> {
        match resource_type {
            ResourceType::Fungible { .. } => {
                Self::compose_by_amount(proofs, None, resource_def_id, resource_type)
            }
            ResourceType::NonFungible => {
                Self::compose_by_ids(proofs, None, resource_def_id, resource_type)
            }
        }
    }

    pub fn compose_by_amount(
        proofs: &[Proof],
        total_amount: Option<Decimal>,
        resource_def_id: ResourceDefId,
        resource_type: ResourceType,
    ) -> Result<Proof, ProofError> {
        // check resource type
        if matches!(resource_type, ResourceType::NonFungible) {
            return Err(ProofError::FungibleOperationNotAllowed);
        }

        // filter proofs by resource def id and restricted flag
        let proofs: Vec<&Proof> = proofs
            .iter()
            .filter(|p| p.resource_def_id() == resource_def_id && !p.is_restricted())
            .collect();

        // calculate the max locked amount (by the input proofs) in each container
        let mut max = HashMap::<ProofSourceId, Decimal>::new();
        for proof in &proofs {
            match proof {
                Proof::Fungible { sources, .. } => {
                    for (source_id, (_, amount)) in sources {
                        if let Some(existing) = max.get_mut(source_id) {
                            *existing = Decimal::max(*existing, amount.clone());
                        } else {
                            max.insert(source_id.clone(), amount.clone());
                        }
                    }
                }
                Proof::NonFungible { .. } => panic!("Illegal state"),
            }
        }

        // check if the max satisfied the requested amount
        let max_sum = max
            .values()
            .cloned()
            .reduce(|a, b| a + b)
            .unwrap_or_default();
        let total_amount = total_amount.unwrap_or(max_sum.clone());
        if total_amount > max_sum {
            return Err(ProofError::InsufficientBaseProofs);
        }

        // lock all relevant resources
        //
        // This is not an efficient way of producing proofs, in terms of number of state updates
        // to the resource containers. However, this is the simplest to explain as no
        // resource container selection algorithm is required. All the ref counts increase by 1.
        //
        // If this turns to be a performance bottleneck, should start with containers where the
        // largest amount has been locked, and only lock the requested amount.
        //
        // Another consideration is that this may not be user expected behavior. Because we're locking
        // all the resources, the `total_amount` may be less than the max sum of the locked resources.
        // When a composite proof is used for a second composite proof, the `total_amount` is not
        // respected, all the underlying resource constraints become visible.
        let mut new_sources = HashMap::new();
        for proof in &proofs {
            match proof {
                Proof::Fungible { sources, .. } => {
                    for (source_id, (container, amount)) in sources {
                        container
                            .borrow_mut()
                            .lock_amount(amount.clone())
                            .map_err(ProofError::ResourceContainerError)
                            .expect("Re-locking should always succeed");
                        new_sources.insert(source_id.clone(), (container.clone(), amount.clone()));
                    }
                }
                Proof::NonFungible { .. } => panic!("Illegal state"),
            }
        }

        // issue a new proof
        Proof::new_fungible(resource_def_id, false, total_amount, new_sources)
    }

    pub fn compose_by_ids(
        proofs: &[Proof],
        total_ids: Option<&BTreeSet<NonFungibleId>>,
        resource_def_id: ResourceDefId,
        resource_type: ResourceType,
    ) -> Result<Proof, ProofError> {
        // check resource type
        if matches!(resource_type, ResourceType::Fungible { .. }) {
            return Err(ProofError::NonFungibleOperationNotAllowed);
        }

        // filter proofs by resource def id and restricted flag
        let proofs: Vec<&Proof> = proofs
            .iter()
            .filter(|p| p.resource_def_id() == resource_def_id && !p.is_restricted())
            .collect();

        // calculate the max locked id set (by the input proofs) in each container
        let mut max = HashMap::<ProofSourceId, BTreeSet<NonFungibleId>>::new();
        for proof in &proofs {
            match proof {
                Proof::NonFungible { sources, .. } => {
                    for (source_id, (_, ids)) in sources {
                        if let Some(ids) = max.get_mut(source_id) {
                            ids.extend(ids.clone());
                        } else {
                            max.insert(source_id.clone(), ids.clone());
                        }
                    }
                }
                Proof::Fungible { .. } => panic!("Illegal state"),
            }
        }

        // check if the max satisfied the requested amount
        let mut max_sum = BTreeSet::<NonFungibleId>::new();
        for (_, value) in max {
            max_sum.extend(value);
        }
        let total_ids = total_ids.unwrap_or(&max_sum);
        if !max_sum.is_superset(&total_ids) {
            return Err(ProofError::InsufficientBaseProofs);
        }

        // lock all relevant resources.
        // See `compose_by_amount` for performance notes.
        let mut new_sources = HashMap::new();
        for proof in &proofs {
            match proof {
                Proof::NonFungible { sources, .. } => {
                    for (source_id, (container, ids)) in sources {
                        container
                            .borrow_mut()
                            .lock_ids(ids)
                            .map_err(ProofError::ResourceContainerError)
                            .expect("Re-locking should always succeed");
                        new_sources.insert(source_id.clone(), (container.clone(), ids.clone()));
                    }
                }
                Proof::Fungible { .. } => panic!("Illegal state"),
            }
        }

        // issue a new proof
        Proof::new_non_fungible(resource_def_id, false, total_ids.clone(), new_sources)
    }

    pub fn clone(&self) -> Self {
        match self {
            Self::Fungible {
                resource_def_id,
                restricted,
                total_amount,
                sources,
            } => {
                for (container, amount) in sources.values() {
                    container
                        .borrow_mut()
                        .lock_amount(amount.clone())
                        .expect("Cloning should always succeed");
                }

                Self::Fungible {
                    resource_def_id: resource_def_id.clone(),
                    restricted: restricted.clone(),
                    total_amount: total_amount.clone(),
                    sources: sources.clone(),
                }
            }
            Self::NonFungible {
                resource_def_id,
                restricted,
                total_ids,
                sources,
            } => {
                for (container, ids) in sources.values() {
                    container
                        .borrow_mut()
                        .lock_ids(ids)
                        .expect("Cloning should always succeed");
                }

                Self::NonFungible {
                    resource_def_id: resource_def_id.clone(),
                    restricted: restricted.clone(),
                    total_ids: total_ids.clone(),
                    sources: sources.clone(),
                }
            }
        }
    }

    pub fn drop(self) {
        match self {
            Self::Fungible { sources, .. } => {
                for (container, amount) in sources.values() {
                    container
                        .borrow_mut()
                        .unlock_amount(amount.clone())
                        .expect("Unlocking should always succeed");
                }
            }
            Self::NonFungible { sources, .. } => {
                for (container, ids) in sources.values() {
                    container
                        .borrow_mut()
                        .unlock_ids(ids)
                        .expect("Unlocking should always succeed");
                }
            }
        }
    }

    pub fn change_to_restricted(&mut self) {
        match self {
            Self::Fungible { restricted, .. } | Self::NonFungible { restricted, .. } => {
                *restricted = true;
            }
        }
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        match self {
            Self::Fungible {
                resource_def_id, ..
            }
            | Self::NonFungible {
                resource_def_id, ..
            } => resource_def_id.clone(),
        }
    }

    pub fn total_amount(&self) -> Decimal {
        match self {
            Self::Fungible { total_amount, .. } => total_amount.clone(),
            Self::NonFungible { total_ids, .. } => total_ids.len().into(),
        }
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleId>, ProofError> {
        match self {
            Self::Fungible { .. } => Err(ProofError::NonFungibleOperationNotAllowed),
            Self::NonFungible { total_ids, .. } => Ok(total_ids.clone()),
        }
    }

    pub fn is_restricted(&self) -> bool {
        match self {
            Self::Fungible { restricted, .. } | Self::NonFungible { restricted, .. } => *restricted,
        }
    }
}
