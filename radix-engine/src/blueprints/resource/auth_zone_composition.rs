use crate::blueprints::resource::*;
use crate::errors::{ApplicationError, RuntimeError};
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::node_init::NodeInit;
use crate::types::*;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::blueprints::resource::*;

use super::AuthZoneError;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ComposeProofError {
    NonFungibleOperationNotSupported,
    InsufficientBaseProofs,
    InvalidAmount,
}

pub enum ComposedProof {
    Fungible(ProofInfoSubstate, FungibleProof),
    NonFungible(ProofInfoSubstate, NonFungibleProof),
}

impl From<ComposedProof> for NodeInit {
    fn from(value: ComposedProof) -> Self {
        match value {
            ComposedProof::Fungible(info, proof) => NodeInit::Object(btreemap!(
                ProofOffset::Info.into() => IndexedScryptoValue::from_typed(&info),
                ProofOffset::Fungible.into() => IndexedScryptoValue::from_typed(&proof),
            )),
            ComposedProof::NonFungible(info, proof) => NodeInit::Object(btreemap!(
                ProofOffset::Info.into() => IndexedScryptoValue::from_typed(&info),
                ProofOffset::NonFungible.into() => IndexedScryptoValue::from_typed(&proof),
            )),
        }
    }
}

/// Compose a proof by amount, given a list of proofs of any address.
pub fn compose_proof_by_amount<Y: KernelSubstateApi + ClientApi<RuntimeError>>(
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
            compose_fungible_proof(proofs, resource_address, amount, api).map(|proof| {
                ComposedProof::Fungible(
                    ProofInfoSubstate {
                        resource_address,
                        resource_type,
                        restricted: false, // TODO: follow existing impl, but need to revisit this
                    },
                    proof,
                )
            })
        }
        ResourceType::NonFungible { .. } => compose_non_fungible_proof(
            proofs,
            resource_address,
            match amount {
                Some(amount) => NonFungiblesSpecification::Some(
                    amount.to_string().parse().expect("Amount checked upfront"),
                ),
                None => NonFungiblesSpecification::All,
            },
            api,
        )
        .map(|proof| {
            ComposedProof::NonFungible(
                ProofInfoSubstate {
                    resource_address,
                    resource_type,
                    restricted: false, // TODO: follow existing impl, but need to revisit this
                },
                proof,
            )
        }),
    }
}

/// Compose a proof by ids, given a list of proofs of any address.
pub fn compose_proof_by_ids<Y: KernelSubstateApi + ClientApi<RuntimeError>>(
    proofs: &[Proof],
    resource_address: ResourceAddress,
    ids: Option<BTreeSet<NonFungibleLocalId>>,
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
        .map(|proof| {
            ComposedProof::NonFungible(
                ProofInfoSubstate {
                    resource_address,
                    resource_type,
                    restricted: false, // TODO: follow existing impl, but need to revisit this
                },
                proof,
            )
        }),
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
        let handle = api.sys_lock_substate(
            proof.0.as_node_id(),
            &ProofOffset::Info.into(),
            LockFlags::read_only(),
        )?;
        let proof_info: ProofInfoSubstate = api.sys_read_substate_typed(handle)?;
        if proof_info.resource_address == resource_address {
            api.sys_drop_lock(handle)?;

            let handle = api.sys_lock_substate(
                proof.0.as_node_id(),
                &ProofOffset::Fungible.into(),
                LockFlags::read_only(),
            )?;
            let proof: FungibleProof = api.sys_read_substate_typed(handle)?;
            for (container, locked_amount) in &proof.evidence {
                if let Some(existing) = max.get_mut(container) {
                    *existing = Decimal::max(*existing, locked_amount.clone());
                } else {
                    max.insert(container.clone(), locked_amount.clone());
                }
            }
            api.sys_drop_lock(handle)?;
        } else {
            api.sys_drop_lock(handle)?;
        }
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
        NonIterMap<LocalRef, BTreeSet<NonFungibleLocalId>>,
    ),
    RuntimeError,
> {
    let mut total = BTreeSet::<NonFungibleLocalId>::new();
    // calculate the max locked non-fungibles of each container
    let mut per_container = NonIterMap::<LocalRef, BTreeSet<NonFungibleLocalId>>::new();
    for proof in proofs {
        let handle = api.sys_lock_substate(
            proof.0.as_node_id(),
            &ProofOffset::Info.into(),
            LockFlags::read_only(),
        )?;
        let proof_info: ProofInfoSubstate = api.sys_read_substate_typed(handle)?;
        if proof_info.resource_address == resource_address {
            api.sys_drop_lock(handle)?;

            let handle = api.sys_lock_substate(
                proof.0.as_node_id(),
                &ProofOffset::NonFungible.into(),
                LockFlags::read_only(),
            )?;
            let proof: NonFungibleProof = api.sys_read_substate_typed(handle)?;
            for (container, locked_ids) in &proof.evidence {
                total.extend(locked_ids.clone());
                if let Some(ids) = per_container.get_mut(container) {
                    ids.extend(locked_ids.clone());
                } else {
                    per_container.insert(container.clone(), locked_ids.clone());
                }
            }
        } else {
            api.sys_drop_lock(handle)?;
        }
    }
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
        let handle = api.sys_lock_substate(
            proof.0.as_node_id(),
            &ProofOffset::Fungible.into(),
            LockFlags::read_only(),
        )?;
        let substate: FungibleProof = api.sys_read_substate_typed(handle)?;
        let proof = substate.clone();
        for (container, _) in &proof.evidence {
            if remaining.is_zero() {
                break 'outer;
            }

            if let Some(quota) = per_container.remove(container) {
                let amount = Decimal::min(remaining, quota);
                api.call_method(
                    container.as_node_id(),
                    match container {
                        LocalRef::Bucket(_) => BUCKET_LOCK_AMOUNT_IDENT,
                        LocalRef::Vault(_) => FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT,
                    },
                    scrypto_args!(amount),
                )?;
                remaining -= amount;
                evidence.insert(container.clone(), amount);
            }
        }
        api.sys_drop_lock(handle)?;
    }

    FungibleProof::new(amount, evidence)
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
        let handle = api.sys_lock_substate(
            proof.0.as_node_id(),
            &ProofOffset::NonFungible.into(),
            LockFlags::read_only(),
        )?;
        let substate: NonFungibleProof = api.sys_read_substate_typed(handle)?;
        let proof = substate.clone();
        for (container, _) in &proof.evidence {
            if remaining.is_empty() {
                break 'outer;
            }

            if let Some(quota) = per_container.remove(container) {
                let ids = remaining.intersection(&quota).cloned().collect();
                api.call_method(
                    container.as_node_id(),
                    match container {
                        LocalRef::Bucket(_) => BUCKET_LOCK_NON_FUNGIBLES_IDENT,
                        LocalRef::Vault(_) => NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT,
                    },
                    scrypto_args!(&ids),
                )?;
                for id in &ids {
                    remaining.remove(id);
                }
                evidence.insert(container.clone(), ids);
            }
        }
        api.sys_drop_lock(handle)?;
    }

    NonFungibleProof::new(ids.clone(), evidence)
        .map_err(|e| RuntimeError::ApplicationError(ApplicationError::ProofError(e)))
}
