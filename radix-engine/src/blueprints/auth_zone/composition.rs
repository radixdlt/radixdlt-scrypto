use crate::blueprints::resource::*;
use crate::errors::{ApplicationError, RuntimeError};
use crate::kernel::kernel_api::{KernelSubstateApi, LockFlags};
use crate::types::*;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;

use super::AuthZoneError;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ComposeProofError {
    NonFungibleOperationNotSupported,
    InsufficientBaseProofs,
    InvalidAmount,
}

pub fn compose_proof_by_amount<Y: KernelSubstateApi + ClientApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    amount: Option<Decimal>,
    api: &mut Y,
) -> Result<ProofSubstate, RuntimeError> {
    let resource_type = ResourceManager(resource_address).resource_type(api)?;

    match resource_type {
        ResourceType::Fungible { divisibility } => {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::AuthZoneError(super::AuthZoneError::ComposeProofError(
                    ComposeProofError::NonFungibleOperationNotSupported,
                )),
            ));
        }
        ResourceType::NonFungible { id_type } => compose_non_fungible_proof(
            proofs,
            resource_address,
            match amount {
                Some(amount) => {
                    if let Ok(n) = amount.to_string().parse() {
                        NonFungiblesSpecification::Some(n)
                    } else {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::AuthZoneError(AuthZoneError::ComposeProofError(
                                ComposeProofError::InvalidAmount,
                            )),
                        ));
                    }
                }
                None => NonFungiblesSpecification::All,
            },
            api,
        )
        .map(ProofSubstate::from),
    }
}

pub fn compose_proof_by_ids<Y: KernelSubstateApi + ClientApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    ids: Option<BTreeSet<NonFungibleLocalId>>,
    api: &mut Y,
) -> Result<ProofSubstate, RuntimeError> {
    let resource_type = ResourceManager(resource_address).resource_type(api)?;

    match resource_type {
        ResourceType::Fungible { divisibility } => {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::AuthZoneError(super::AuthZoneError::ComposeProofError(
                    ComposeProofError::NonFungibleOperationNotSupported,
                )),
            ));
        }
        ResourceType::NonFungible { id_type } => compose_non_fungible_proof(
            proofs,
            resource_address,
            match ids {
                Some(ids) => NonFungiblesSpecification::Exact(ids),
                None => NonFungiblesSpecification::All,
            },
            api,
        )
        .map(ProofSubstate::from),
    }
}

//====================
// Helper functions
//====================

fn max_amount_locked<Y: KernelSubstateApi + ClientApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    api: &mut Y,
) -> Result<(Decimal, BTreeMap<LocalRef, Decimal>), RuntimeError> {
    // calculate the max locked amount of each container
    let mut max = BTreeMap::<LocalRef, Decimal>::new();
    for proof in proofs {
        let handle = api.kernel_lock_substate(
            RENodeId::Proof(proof.0),
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate = api.kernel_get_substate_ref(handle)?;
        let proof_substate = substate.proof();
        if proof_substate.resource_address() == resource_address {
            if let ProofSubstate::Fungible(f) = proof_substate {
                for (container_id, locked_amount) in &f.evidence {
                    if let Some(existing) = max.get_mut(container_id) {
                        *existing = Decimal::max(*existing, locked_amount.clone());
                    } else {
                        max.insert(container_id.clone(), locked_amount.clone());
                    }
                }
            }
        }
        api.kernel_drop_lock(handle)?;
    }
    let total = max
        .values()
        .cloned()
        .reduce(|a, b| a + b)
        .unwrap_or_default();
    let per_container = max.into_iter().collect();
    Ok((total, per_container))
}

fn max_ids_locked<Y: KernelSubstateApi + ClientApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    api: &mut Y,
) -> Result<
    (
        BTreeSet<NonFungibleLocalId>,
        HashMap<LocalRef, BTreeSet<NonFungibleLocalId>>,
    ),
    RuntimeError,
> {
    // calculate the max locked non-fungibles of each container
    let mut max = HashMap::<LocalRef, BTreeSet<NonFungibleLocalId>>::new();
    for proof in proofs {
        let handle = api.kernel_lock_substate(
            RENodeId::Proof(proof.0),
            NodeModuleId::SELF,
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let substate = api.kernel_get_substate_ref(handle)?;
        let proof_substate = substate.proof();
        if proof_substate.resource_address() == resource_address {
            if let ProofSubstate::NonFungible(nf) = proof_substate {
                for (container_id, locked_ids) in &nf.evidence {
                    let new_ids = locked_ids.clone();
                    if let Some(ids) = max.get_mut(container_id) {
                        ids.extend(new_ids);
                    } else {
                        max.insert(container_id.clone(), new_ids);
                    }
                }
            }
        }
        api.kernel_drop_lock(handle)?;
    }
    let mut total = BTreeSet::<NonFungibleLocalId>::new();
    for value in max.values() {
        total.extend(value.clone());
    }
    let per_container = max.into_iter().collect();
    Ok((total, per_container))
}

fn compose_fungible_proof<Y: KernelSubstateApi + ClientApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    amount: Option<Decimal>,
    api: &mut Y,
) -> Result<FungibleProof, RuntimeError> {
    let (max_locked, mut per_container) = max_amount_locked(proofs, resource_address, api)?;
    let amount = amount.unwrap_or(max_locked);

    // Check if base proofs are sufficient for the request amount
    if amount > max_locked {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::AuthZoneError(AuthZoneError::ComposeProofError(
                ComposeProofError::InsufficientBaseProofs,
            )),
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

enum NonFungiblesSpecification {
    All,
    Some(usize),
    Exact(BTreeSet<NonFungibleLocalId>),
}

fn compose_non_fungible_proof<Y: KernelSubstateApi + ClientApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    ids: NonFungiblesSpecification,
    api: &mut Y,
) -> Result<NonFungibleProof, RuntimeError> {
    let (max_locked, mut per_container) = max_ids_locked(proofs, resource_address, api)?;
    let ids = match ids {
        NonFungiblesSpecification::All => max_locked.clone(),
        NonFungiblesSpecification::Some(n) => {
            let ids: BTreeSet<NonFungibleLocalId> = max_locked.iter().cloned().take(n).collect();
            if ids.len() != n {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::AuthZoneError(AuthZoneError::ComposeProofError(
                        ComposeProofError::InsufficientBaseProofs,
                    )),
                ));
            }
            ids
        }
        NonFungiblesSpecification::Exact(ids) => {
            if !max_locked.is_superset(&ids) {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::AuthZoneError(AuthZoneError::ComposeProofError(
                        ComposeProofError::InsufficientBaseProofs,
                    )),
                ));
            }
            ids
        }
    };

    if !max_locked.is_superset(&ids) {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::AuthZoneError(AuthZoneError::ComposeProofError(
                ComposeProofError::InsufficientBaseProofs,
            )),
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
