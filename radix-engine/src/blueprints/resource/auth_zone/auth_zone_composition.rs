use crate::blueprints::resource::*;
use crate::errors::{ApplicationError, RuntimeError};
use crate::internal_prelude::*;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::system_callback::SystemLockData;
use crate::system::system_substates::FieldSubstate;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::resource::*;
use radix_native_sdk::resource::ResourceManager;

use super::AuthZoneError;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ComposeProofError {
    NonFungibleOperationNotSupported,
    InsufficientBaseProofs,
    InvalidAmount,
    UnexpectedDecimalComputationError,
}

pub enum ComposedProof {
    Fungible(
        ProofMoveableSubstate,
        FungibleProofSubstate,
        Vec<SubstateHandle>,
    ),
    NonFungible(
        ProofMoveableSubstate,
        NonFungibleProofSubstate,
        Vec<SubstateHandle>,
    ),
}

impl From<ComposedProof> for BTreeMap<SubstateKey, IndexedScryptoValue> {
    fn from(value: ComposedProof) -> Self {
        match value {
            ComposedProof::Fungible(info, proof, ..) => btreemap!(
                FungibleProofField::Moveable.into() => IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(info)),
                FungibleProofField::ProofRefs.into() => IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(proof)),
            ),
            ComposedProof::NonFungible(info, proof, ..) => btreemap!(
                NonFungibleProofField::Moveable.into() => IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(info)),
                NonFungibleProofField::ProofRefs.into() => IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(proof)),
            ),
        }
    }
}

/// Compose a proof by amount, given a list of proofs of any address.
pub fn compose_proof_by_amount<Y: KernelSubstateApi<SystemLockData> + SystemApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    amount: Option<Decimal>,
    api: &mut Y,
) -> Result<ComposedProof, RuntimeError> {
    let resource_type = ResourceManager(resource_address).resource_type(api)?;

    if let Some(amount) = amount {
        if !resource_type.check_amount(amount) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::AuthZoneError(AuthZoneError::ComposeProofError(
                    ComposeProofError::InvalidAmount,
                )),
            ));
        }
    }

    match resource_type {
        ResourceType::Fungible { .. } => {
            compose_fungible_proof(proofs, resource_address, amount, api).map(|(proof, handles)| {
                ComposedProof::Fungible(ProofMoveableSubstate { restricted: false }, proof, handles)
            })
        }
        ResourceType::NonFungible { .. } => compose_non_fungible_proof(
            proofs,
            resource_address,
            match amount {
                Some(amount) => {
                    NonFungiblesSpecification::Some(amount.to_string().parse().map_err(|_| {
                        RuntimeError::ApplicationError(ApplicationError::AuthZoneError(
                            AuthZoneError::ComposeProofError(ComposeProofError::InvalidAmount),
                        ))
                    })?)
                }
                None => NonFungiblesSpecification::All,
            },
            api,
        )
        .map(|(proof, handles)| {
            ComposedProof::NonFungible(ProofMoveableSubstate { restricted: false }, proof, handles)
        }),
    }
}

/// Compose a proof by ids, given a list of proofs of any address.
pub fn compose_proof_by_ids<Y: KernelSubstateApi<SystemLockData> + SystemApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    ids: Option<IndexSet<NonFungibleLocalId>>,
    api: &mut Y,
) -> Result<ComposedProof, RuntimeError> {
    let resource_type = ResourceManager(resource_address).resource_type(api)?;

    match resource_type {
        ResourceType::Fungible { .. } => {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::AuthZoneError(super::AuthZoneError::ComposeProofError(
                    ComposeProofError::NonFungibleOperationNotSupported,
                )),
            ));
        }
        ResourceType::NonFungible { .. } => compose_non_fungible_proof(
            proofs,
            resource_address,
            match ids {
                Some(ids) => NonFungiblesSpecification::Exact(ids),
                None => NonFungiblesSpecification::All,
            },
            api,
        )
        .map(|(proof, handles)| {
            ComposedProof::NonFungible(ProofMoveableSubstate { restricted: false }, proof, handles)
        }),
    }
}

//====================
// Helper functions
//====================

fn max_amount_locked<Y: KernelSubstateApi<SystemLockData> + SystemApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    api: &mut Y,
) -> Result<(Decimal, IndexMap<LocalRef, Decimal>), RuntimeError> {
    // calculate the max locked amount of each container
    let mut max: IndexMap<LocalRef, Decimal> = index_map_new();
    for proof in proofs {
        let blueprint_id = api.get_blueprint_id(proof.0.as_node_id())?;

        if blueprint_id.blueprint_name.eq(FUNGIBLE_PROOF_BLUEPRINT) {
            let outer_object = api.get_outer_object(proof.0.as_node_id())?;
            let proof_resource = ResourceAddress::new_or_panic(outer_object.into());
            if proof_resource == resource_address {
                let handle = api.kernel_open_substate(
                    proof.0.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &FungibleProofField::ProofRefs.into(),
                    LockFlags::read_only(),
                    SystemLockData::default(),
                )?;
                let proof: FieldSubstate<FungibleProofSubstate> =
                    api.kernel_read_substate(handle)?.as_typed().unwrap();
                for (container, locked_amount) in &proof.into_payload().evidence {
                    if let Some(existing) = max.get_mut(container) {
                        *existing = Decimal::max(*existing, locked_amount.clone());
                    } else {
                        max.insert(container.clone(), locked_amount.clone());
                    }
                }
                api.kernel_close_substate(handle)?;
            }
        }
    }

    let mut total = Decimal::ZERO;
    for v in max.values().cloned() {
        total = total.checked_add(v).ok_or(RuntimeError::ApplicationError(
            ApplicationError::AuthZoneError(AuthZoneError::ComposeProofError(
                ComposeProofError::UnexpectedDecimalComputationError,
            )),
        ))?;
    }
    let per_container = max.into_iter().collect();
    Ok((total, per_container))
}

fn max_ids_locked<Y: KernelSubstateApi<SystemLockData> + SystemApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    api: &mut Y,
) -> Result<
    (
        IndexSet<NonFungibleLocalId>,
        NonIterMap<LocalRef, IndexSet<NonFungibleLocalId>>,
    ),
    RuntimeError,
> {
    let mut total: IndexSet<NonFungibleLocalId> = index_set_new();
    // calculate the max locked non-fungibles of each container
    let mut per_container = NonIterMap::<LocalRef, IndexSet<NonFungibleLocalId>>::new();
    for proof in proofs {
        let blueprint_id = api.get_blueprint_id(proof.0.as_node_id())?;
        if blueprint_id.blueprint_name.eq(NON_FUNGIBLE_PROOF_BLUEPRINT) {
            let outer_object = api.get_outer_object(proof.0.as_node_id())?;
            let proof_resource = ResourceAddress::new_or_panic(outer_object.into());
            if proof_resource == resource_address {
                let handle = api.kernel_open_substate(
                    proof.0.as_node_id(),
                    MAIN_BASE_PARTITION,
                    &NonFungibleProofField::ProofRefs.into(),
                    LockFlags::read_only(),
                    SystemLockData::default(),
                )?;
                let proof: FieldSubstate<NonFungibleProofSubstate> =
                    api.kernel_read_substate(handle)?.as_typed().unwrap();
                for (container, locked_ids) in &proof.into_payload().evidence {
                    total.extend(locked_ids.clone());
                    if let Some(ids) = per_container.get_mut(container) {
                        ids.extend(locked_ids.clone());
                    } else {
                        per_container.insert(container.clone(), locked_ids.clone());
                    }
                }
            }
        }
    }
    Ok((total, per_container))
}

fn compose_fungible_proof<Y: KernelSubstateApi<SystemLockData> + SystemApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    amount: Option<Decimal>,
    api: &mut Y,
) -> Result<(FungibleProofSubstate, Vec<SubstateHandle>), RuntimeError> {
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

    let mut evidence = index_map_new();
    let mut remaining = amount.clone();
    let mut lock_handles = Vec::new();
    'outer: for proof in proofs {
        let handle = api.kernel_open_substate(
            proof.0.as_node_id(),
            MAIN_BASE_PARTITION,
            &FungibleProofField::ProofRefs.into(),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;
        let substate: FieldSubstate<FungibleProofSubstate> =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        let proof = substate.into_payload();
        for (container, _) in &proof.evidence {
            if remaining.is_zero() {
                break 'outer;
            }

            if let Some(quota) = per_container.swap_remove(container) {
                let amount = Decimal::min(remaining, quota);
                api.call_method(
                    container.as_node_id(),
                    match container {
                        LocalRef::Bucket(_) => FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT,
                        LocalRef::Vault(_) => FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT,
                    },
                    scrypto_args!(amount),
                )?;
                remaining = remaining
                    .checked_sub(amount)
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::AuthZoneError(AuthZoneError::ComposeProofError(
                            ComposeProofError::UnexpectedDecimalComputationError,
                        )),
                    ))?;
                evidence.insert(container.clone(), amount);
            }
        }
        lock_handles.push(handle);
    }

    Ok((
        FungibleProofSubstate::new(amount, evidence)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::ProofError(e)))?,
        lock_handles,
    ))
}

enum NonFungiblesSpecification {
    All,
    Some(usize),
    Exact(IndexSet<NonFungibleLocalId>),
}

fn compose_non_fungible_proof<Y: KernelSubstateApi<SystemLockData> + SystemApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    ids: NonFungiblesSpecification,
    api: &mut Y,
) -> Result<(NonFungibleProofSubstate, Vec<SubstateHandle>), RuntimeError> {
    let (max_locked, mut per_container) = max_ids_locked(proofs, resource_address, api)?;
    let ids = match ids {
        NonFungiblesSpecification::All => max_locked.clone(),
        NonFungiblesSpecification::Some(n) => {
            let ids: IndexSet<NonFungibleLocalId> = max_locked.iter().cloned().take(n).collect();
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

    let mut evidence = index_map_new();
    let mut remaining = ids.clone();
    let mut lock_handles = Vec::new();
    'outer: for proof in proofs {
        let handle = api.kernel_open_substate(
            proof.0.as_node_id(),
            MAIN_BASE_PARTITION,
            &NonFungibleProofField::ProofRefs.into(),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;
        let substate: FieldSubstate<NonFungibleProofSubstate> =
            api.kernel_read_substate(handle)?.as_typed().unwrap();
        let proof = substate.into_payload().clone();
        for (container, _) in &proof.evidence {
            if remaining.is_empty() {
                break 'outer;
            }

            if let Some(quota) = per_container.remove(container) {
                let ids = remaining.intersection(&quota).cloned().collect();
                api.call_method(
                    container.as_node_id(),
                    match container {
                        LocalRef::Bucket(_) => NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT,
                        LocalRef::Vault(_) => NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT,
                    },
                    scrypto_args!(&ids),
                )?;
                for id in &ids {
                    remaining.swap_remove(id);
                }
                evidence.insert(container.clone(), ids);
            }
        }
        lock_handles.push(handle);
    }

    Ok((
        NonFungibleProofSubstate::new(ids.clone(), evidence)
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::ProofError(e)))?,
        lock_handles,
    ))
}
