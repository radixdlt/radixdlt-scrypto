use crate::blueprints::resource::*;
use crate::errors::{ApplicationError, RuntimeError};
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;

fn compute_max_amount_locked(
    proofs: &[FungibleProof],
    resource_address: ResourceAddress,
) -> (Decimal, BTreeMap<LocalRef, Decimal>) {
    // filter proofs by resource address and restricted flag
    let proofs: Vec<&FungibleProof> = proofs
        .iter()
        .filter(|p| p.resource_address() == resource_address && !p.is_restricted())
        .collect();

    // calculate the max locked amount of each container
    let mut max = BTreeMap::<LocalRef, Decimal>::new();
    for proof in &proofs {
        for (container_id, locked_amount) in &proof.evidence {
            if let Some(existing) = max.get_mut(container_id) {
                *existing = Decimal::max(*existing, locked_amount.clone());
            } else {
                max.insert(container_id.clone(), locked_amount.clone());
            }
        }
    }
    let total = max
        .values()
        .cloned()
        .reduce(|a, b| a + b)
        .unwrap_or_default();
    let per_container = max.into_iter().collect();
    (total, per_container)
}

pub fn compose_fungible_proof_by_amount<Y: ClientApi<RuntimeError>>(
    proofs: &[FungibleProof],
    resource_address: ResourceAddress,
    amount: Option<Decimal>,
    api: &mut Y,
) -> Result<FungibleProof, RuntimeError> {
    let (total_locked, mut per_container) = compute_max_amount_locked(proofs, resource_address);
    let amount = amount.unwrap_or(total_locked);

    // Check if base proofs are sufficient for the request amount
    if amount > total_locked {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::ProofError(ProofError::InsufficientBaseProofs),
        ));
    }

    // TODO: review resource container selection algorithm here
    let mut evidence = BTreeMap::new();
    let mut remaining = amount.clone();
    'outer: for proof in proofs {
        for (container_id, _) in &proof.evidence {
            if remaining.is_zero() {
                break 'outer;
            }

            if let Some(quota) = per_container.remove(container_id) {
                let amount = Decimal::min(remaining, quota);
                api.call_method(
                    container_id.to_re_node_id(),
                    match container_id {
                        LocalRef::Bucket(_) => BUCKET_LOCK_AMOUNT_IDENT,
                        LocalRef::Vault(_) => VAULT_LOCK_AMOUNT_IDENT,
                    },
                    scrypto_args!(amount),
                )?;
                remaining -= amount;
                evidence.insert(container_id.clone(), amount);
            }
        }
    }

    FungibleProof::new(resource_address, amount, evidence)
        .map_err(|e| RuntimeError::ApplicationError(ApplicationError::ProofError(e)))
}

pub fn compute_max_ids_locked(
    proofs: &[NonFungibleProof],
    resource_address: ResourceAddress,
) -> (
    BTreeSet<NonFungibleLocalId>,
    HashMap<LocalRef, BTreeSet<NonFungibleLocalId>>,
) {
    // filter proofs by resource address and restricted flag
    let proofs: Vec<&NonFungibleProof> = proofs
        .iter()
        .filter(|p| p.resource_address() == resource_address && !p.is_restricted())
        .collect();

    // calculate the max locked amount (or ids) of each container
    let mut max = HashMap::<LocalRef, BTreeSet<NonFungibleLocalId>>::new();
    for proof in &proofs {
        for (container_id, locked_ids) in &proof.evidence {
            let new_ids = locked_ids.clone();
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
    let per_container = max.into_iter().collect();
    (total, per_container)
}

pub fn compose_non_fungible_proof_by_amount<Y: ClientApi<RuntimeError>>(
    proofs: &[NonFungibleProof],
    resource_address: ResourceAddress,
    amount: Option<Decimal>,
    api: &mut Y,
) -> Result<NonFungibleProof, RuntimeError> {
    let (total_locked, mut per_container) = compute_max_ids_locked(proofs, resource_address);
    let total_amount = total_locked.len().into();
    let amount = amount.unwrap_or(total_amount);

    if amount > total_amount {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::ProofError(ProofError::InsufficientBaseProofs),
        ));
    }

    let n: usize = amount
        .to_string()
        .parse()
        .expect("Failed to convert non-fungible amount to usize");
    let ids: BTreeSet<NonFungibleLocalId> = total_locked.iter().take(n).cloned().collect();
    compose_non_fungible_proof_by_ids(proofs, resource_address, Some(ids), api)
}

pub fn compose_non_fungible_proof_by_ids<Y: ClientApi<RuntimeError>>(
    proofs: &[NonFungibleProof],
    resource_address: ResourceAddress,
    ids: Option<BTreeSet<NonFungibleLocalId>>,
    api: &mut Y,
) -> Result<NonFungibleProof, RuntimeError> {
    let (total_locked, mut per_container) = compute_max_ids_locked(proofs, resource_address);
    let ids = ids.unwrap_or(total_locked.clone());

    if !total_locked.is_superset(&ids) {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::ProofError(ProofError::InsufficientBaseProofs),
        ));
    }

    // TODO: review resource container selection algorithm here
    let mut evidence = BTreeMap::new();
    let mut remaining = ids.clone();
    'outer: for proof in proofs {
        for (container_id, _) in &proof.evidence {
            if remaining.is_empty() {
                break 'outer;
            }

            if let Some(quota) = per_container.remove(container_id) {
                let ids = remaining.intersection(&quota).cloned().collect();
                api.call_method(
                    container_id.to_re_node_id(),
                    match container_id {
                        LocalRef::Bucket(_) => BUCKET_LOCK_NON_FUNGIBLES_IDENT,
                        LocalRef::Vault(_) => VAULT_LOCK_NON_FUNGIBLES_IDENT,
                    },
                    scrypto_args!(&ids),
                )?;
                for id in &ids {
                    remaining.remove(id);
                }
                evidence.insert(container_id.clone(), ids);
            }
        }
    }

    NonFungibleProof::new(resource_address, ids.clone(), evidence)
        .map_err(|e| RuntimeError::ApplicationError(ApplicationError::ProofError(e)))
}
