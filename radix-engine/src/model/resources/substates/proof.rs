use crate::model::{
    InvokeError, LockableResource, LockedAmountOrIds, ProofError, ProofSnapshot,
    ResourceContainerId,
};
use crate::types::*;

#[derive(Debug)]
pub struct ProofSubstate {
    /// The resource address.
    pub resource_address: ResourceAddress,
    /// The resource type.
    pub resource_type: ResourceType,
    /// Whether movement of this proof is restricted.
    pub restricted: bool,
    /// The total locked amount or non-fungible ids.
    pub total_locked: LockedAmountOrIds,
    /// The supporting containers.
    pub evidence: HashMap<ResourceContainerId, (Rc<RefCell<LockableResource>>, LockedAmountOrIds)>,
}

impl ProofSubstate {
    pub fn new(
        resource_address: ResourceAddress,
        resource_type: ResourceType,
        total_locked: LockedAmountOrIds,
        evidence: HashMap<ResourceContainerId, (Rc<RefCell<LockableResource>>, LockedAmountOrIds)>,
    ) -> Result<ProofSubstate, ProofError> {
        if total_locked.is_empty() {
            return Err(ProofError::EmptyProofNotAllowed);
        }

        Ok(Self {
            resource_address,
            resource_type,
            restricted: false,
            total_locked,
            evidence,
        })
    }

    /// Computes the locked amount or non-fungible IDs, in total and per resource container.
    pub fn compute_total_locked(
        proofs: &[ProofSubstate],
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> (
        LockedAmountOrIds,
        HashMap<ResourceContainerId, LockedAmountOrIds>,
    ) {
        // filter proofs by resource address and restricted flag
        let proofs: Vec<&ProofSubstate> = proofs
            .iter()
            .filter(|p| p.resource_address() == resource_address && !p.is_restricted())
            .collect();

        // calculate the max locked amount (or ids) of each container
        match resource_type {
            ResourceType::Fungible { .. } => {
                let mut max = HashMap::<ResourceContainerId, Decimal>::new();
                for proof in &proofs {
                    for (container_id, (_, locked_amount_or_ids)) in &proof.evidence {
                        let new_amount = locked_amount_or_ids.amount();
                        if let Some(existing) = max.get_mut(container_id) {
                            *existing = Decimal::max(*existing, new_amount);
                        } else {
                            max.insert(container_id.clone(), new_amount);
                        }
                    }
                }
                let total = max
                    .values()
                    .cloned()
                    .reduce(|a, b| a + b)
                    .unwrap_or_default();
                let per_container = max
                    .into_iter()
                    .map(|(k, v)| (k, LockedAmountOrIds::Amount(v)))
                    .collect();
                (LockedAmountOrIds::Amount(total), per_container)
            }
            ResourceType::NonFungible { .. } => {
                let mut max = HashMap::<ResourceContainerId, BTreeSet<NonFungibleLocalId>>::new();
                for proof in &proofs {
                    for (container_id, (_, locked_amount_or_ids)) in &proof.evidence {
                        let new_ids = locked_amount_or_ids
                            .ids()
                            .expect("Failed to list non-fungible IDS on non-fungible proof");
                        if let Some(ids) = max.get_mut(container_id) {
                            ids.extend(new_ids);
                        } else {
                            max.insert(container_id.clone(), new_ids);
                        }
                    }
                }
                let mut total = BTreeSet::<NonFungibleLocalId>::new();
                for value in max.values() {
                    total.extend(value.clone());
                }
                let per_container = max
                    .into_iter()
                    .map(|(k, v)| (k, LockedAmountOrIds::Ids(v)))
                    .collect();
                (LockedAmountOrIds::Ids(total), per_container)
            }
        }
    }

    /// Creates a composite proof from proofs. This method will generate a max proof.
    pub fn compose(
        proofs: &[ProofSubstate],
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<ProofSubstate, ProofError> {
        let (total, _) = Self::compute_total_locked(proofs, resource_address, resource_type);
        match total {
            LockedAmountOrIds::Amount(amount) => {
                Self::compose_by_amount(proofs, amount, resource_address, resource_type)
            }
            LockedAmountOrIds::Ids(ids) => {
                Self::compose_by_ids(proofs, &ids, resource_address, resource_type)
            }
        }
    }

    pub fn compose_by_amount(
        proofs: &[ProofSubstate],
        amount: Decimal,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<ProofSubstate, ProofError> {
        let (total_locked, mut per_container) =
            Self::compute_total_locked(proofs, resource_address, resource_type);

        match total_locked {
            LockedAmountOrIds::Amount(locked_amount) => {
                if amount > locked_amount {
                    return Err(ProofError::InsufficientBaseProofs);
                }

                // Locked the max (or needed) amount from the containers, in the order that the containers were referenced.
                // TODO: observe the performance/feedback of this container selection algorithm and decide next steps
                let mut evidence = HashMap::new();
                let mut remaining = amount.clone();
                'outer: for proof in proofs {
                    for (container_id, (container, _)) in &proof.evidence {
                        if remaining.is_zero() {
                            break 'outer;
                        }

                        if let Some(quota) = per_container.remove(container_id) {
                            let amount = Decimal::min(remaining, quota.amount());
                            container
                                .borrow_mut()
                                .lock_by_amount(amount)
                                .map_err(ProofError::ResourceOperationError)?;
                            remaining -= amount;
                            evidence.insert(
                                container_id.clone(),
                                (container.clone(), LockedAmountOrIds::Amount(amount)),
                            );
                        }
                    }
                }

                ProofSubstate::new(
                    resource_address,
                    resource_type,
                    LockedAmountOrIds::Amount(amount),
                    evidence,
                )
            }
            LockedAmountOrIds::Ids(locked_ids) => {
                if amount > locked_ids.len().into() {
                    Err(ProofError::InsufficientBaseProofs)
                } else {
                    let n: usize = amount
                        .to_string()
                        .parse()
                        .expect("Failed to convert non-fungible amount to usize");
                    let ids: BTreeSet<NonFungibleLocalId> =
                        locked_ids.iter().take(n).cloned().collect();
                    Self::compose_by_ids(proofs, &ids, resource_address, resource_type)
                }
            }
        }
    }

    pub fn compose_by_ids(
        proofs: &[ProofSubstate],
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        resource_type: ResourceType,
    ) -> Result<ProofSubstate, ProofError> {
        let (total_locked, mut per_container) =
            Self::compute_total_locked(proofs, resource_address, resource_type);

        match total_locked {
            LockedAmountOrIds::Amount(_) => Err(ProofError::NonFungibleOperationNotAllowed),
            LockedAmountOrIds::Ids(locked_ids) => {
                if !locked_ids.is_superset(ids) {
                    return Err(ProofError::InsufficientBaseProofs);
                }

                // Locked the max (or needed) ids from the containers, in the order that the containers were referenced.
                // TODO: observe the performance/feedback of this container selection algorithm and decide next steps
                let mut evidence = HashMap::new();
                let mut remaining = ids.clone();
                'outer: for proof in proofs {
                    for (container_id, (container, _)) in &proof.evidence {
                        if remaining.is_empty() {
                            break 'outer;
                        }

                        if let Some(quota) = per_container.remove(container_id) {
                            let ids = remaining
                                .intersection(&quota.ids().expect(
                                    "Failed to list non-fungible ids on non-fungible resource",
                                ))
                                .cloned()
                                .collect();
                            container
                                .borrow_mut()
                                .lock_by_ids(&ids)
                                .map_err(ProofError::ResourceOperationError)?;
                            for id in &ids {
                                remaining.remove(id);
                            }
                            evidence.insert(
                                container_id.clone(),
                                (container.clone(), LockedAmountOrIds::Ids(ids)),
                            );
                        }
                    }
                }

                ProofSubstate::new(
                    resource_address,
                    resource_type,
                    LockedAmountOrIds::Ids(ids.clone()),
                    evidence,
                )
            }
        }
    }

    /// Makes a clone of this proof.
    ///
    /// Note that cloning a proof will update the ref count of the locked
    /// resources in the source containers.
    pub fn clone(&self) -> Self {
        for (_, (container, locked_amount_or_ids)) in &self.evidence {
            match locked_amount_or_ids {
                LockedAmountOrIds::Amount(amount) => {
                    container
                        .borrow_mut()
                        .lock_by_amount(*amount)
                        .expect("Failed to clone a proof");
                }
                LockedAmountOrIds::Ids(ids) => {
                    container
                        .borrow_mut()
                        .lock_by_ids(ids)
                        .expect("Failed to clone a proof");
                }
            }
        }
        Self {
            resource_address: self.resource_address.clone(),
            resource_type: self.resource_type.clone(),
            restricted: self.restricted,
            total_locked: self.total_locked.clone(),
            evidence: self.evidence.clone(),
        }
    }

    pub fn drop(&mut self) {
        for (_, (container, locked_amount_or_ids)) in &mut self.evidence {
            container.borrow_mut().unlock(locked_amount_or_ids);
        }
    }

    pub fn change_to_unrestricted(&mut self) {
        self.restricted = false;
    }

    pub fn change_to_restricted(&mut self) {
        self.restricted = true;
    }

    pub fn resource_address(&self) -> ResourceAddress {
        self.resource_address
    }

    pub fn total_amount(&self) -> Decimal {
        self.total_locked.amount()
    }

    pub fn total_ids(&self) -> Result<BTreeSet<NonFungibleLocalId>, InvokeError<ProofError>> {
        self.total_locked
            .ids()
            .map_err(|_| InvokeError::SelfError(ProofError::NonFungibleOperationNotAllowed))
    }

    pub fn is_restricted(&self) -> bool {
        self.restricted
    }

    pub fn snapshot(&self) -> ProofSnapshot {
        ProofSnapshot {
            resource_address: self.resource_address,
            resource_type: self.resource_type,
            restricted: self.restricted,
            total_locked: self.total_locked.clone(),
        }
    }
}
